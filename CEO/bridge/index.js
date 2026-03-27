#!/usr/bin/env node
import 'dotenv/config';

/**
 * DAL CEO Bridge
 * Only YOUR messages are forwarded to the agent (prompts/tasks from you).
 * Messages from everyone else are ignored. You give the agent work; the agent
 * does not reply to other people.
 *
 * Prerequisites:
 * - Run the agent first: dal serve server.dal --port 4040  (or dal agent serve --port 4040)
 *
 * Env (required for each channel you use):
 * - AGENT_URL = http://localhost:4040  (default)
 * - TELEGRAM_BOT_TOKEN + ALLOWED_TELEGRAM_IDS = your Telegram user ID(s)
 * - DISCORD_BOT_TOKEN + ALLOWED_DISCORD_IDS = your Discord user ID(s)
 * - ALLOWED_IMESSAGE_HANDLES = your phone/Apple ID (Mac only), e.g. +15551234567
 *
 * Usage:
 *   node index.js              # run all enabled
 *   node index.js --telegram   # Telegram only
 *   node index.js --discord    # Discord only
 *   node index.js --imessage   # iMessage only (Mac)
 */

const AGENT_URL = process.env.AGENT_URL || 'http://localhost:4040';

function parseList(envVar) {
  const raw = process.env[envVar] || '';
  return raw.split(',').map((s) => s.trim()).filter(Boolean);
}

const ALLOWED_TELEGRAM_IDS = new Set(parseList('ALLOWED_TELEGRAM_IDS').map(Number).filter((n) => !Number.isNaN(n)));
const ALLOWED_DISCORD_IDS = new Set(parseList('ALLOWED_DISCORD_IDS').concat(process.env.DISCORD_MY_ID || '').filter(Boolean));
const ALLOWED_IMESSAGE_HANDLES = new Set(parseList('ALLOWED_IMESSAGE_HANDLES').concat(process.env.IMESSAGE_MY_HANDLES || '').filter(Boolean));

async function sendToAgent(senderId, content) {
  const res = await fetch(`${AGENT_URL}/api/message`, {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({ sender_id: senderId, content }),
  });
  if (!res.ok) {
    const t = await res.text();
    throw new Error(`Agent returned ${res.status}: ${t}`);
  }
  const data = await res.json();
  return data.reply ?? data.error ?? 'No reply from agent.';
}

function runTelegram() {
  const token = process.env.TELEGRAM_BOT_TOKEN;
  if (!token) {
    console.warn('TELEGRAM_BOT_TOKEN not set; skipping Telegram.');
    return;
  }
  const TelegramBot = require('node-telegram-bot-api');
  const bot = new TelegramBot(token, { polling: true });
  if (ALLOWED_TELEGRAM_IDS.size === 0) {
    console.warn('ALLOWED_TELEGRAM_IDS not set — no Telegram messages will be forwarded. Set to your Telegram user ID (e.g. from @userinfobot).');
    return;
  }
  bot.on('message', async (msg) => {
    const fromId = msg.from?.id;
    if (fromId == null || !ALLOWED_TELEGRAM_IDS.has(fromId)) return;
    const chatId = msg.chat.id;
    const text = msg.text;
    if (!text) return;
    try {
      const reply = await sendToAgent(`telegram_${fromId}`, text);
      await bot.sendMessage(chatId, reply);
    } catch (e) {
      await bot.sendMessage(chatId, `Error: ${e.message}`);
    }
  });
  console.log('Telegram bridge running — only your messages go to the agent (forwarding to %s)', AGENT_URL);
}

function runDiscord() {
  const token = process.env.DISCORD_BOT_TOKEN;
  if (!token) {
    console.warn('DISCORD_BOT_TOKEN not set; skipping Discord.');
    return;
  }
  const { Client, GatewayIntentBits } = require('discord.js');
  const client = new Client({
    intents: [
      GatewayIntentBits.Guilds,
      GatewayIntentBits.GuildMessages,
      GatewayIntentBits.MessageContent,
      GatewayIntentBits.DirectMessages,
    ],
  });
  client.once('ready', () => {
    console.log('Discord bridge running as %s (forwarding to %s)', client.user.tag, AGENT_URL);
  });
  if (ALLOWED_DISCORD_IDS.size === 0) {
    console.warn('ALLOWED_DISCORD_IDS not set — no Discord messages will be forwarded. Set to your Discord user ID (Developer Mode → right-click you → Copy ID).');
    return;
  }
  client.on('messageCreate', async (message) => {
    if (message.author.bot) return;
    const authorId = message.author.id;
    if (!ALLOWED_DISCORD_IDS.has(authorId)) return;
    const content = message.content?.trim();
    if (!content) return;
    const senderId = `discord_${authorId}`;
    try {
      const reply = await sendToAgent(senderId, content);
      await message.reply(reply);
    } catch (e) {
      await message.reply(`Error: ${e.message}`);
    }
  });
  client.login(token);
}

async function runImessage() {
  if (process.platform !== 'darwin') {
    console.warn('iMessage bridge is only supported on macOS. Skipping.');
    return;
  }
  const { createRequire } = await import('module');
  const require = createRequire(import.meta.url);
  let imessage;
  try {
    imessage = require('osa-imessage');
  } catch (e) {
    console.warn('osa-imessage not available:', e.message, '- run npm install');
    return;
  }
  const allowedHandles = ALLOWED_IMESSAGE_HANDLES;
  if (allowedHandles.size === 0) {
    console.warn('ALLOWED_IMESSAGE_HANDLES not set — no iMessages will be forwarded. Set to your phone number or Apple ID (e.g. +15551234567).');
    return;
  }
  imessage.listen().on('message', async (msg) => {
    const isFromYou = msg.fromMe || allowedHandles.has(String(msg.handle).trim());
    if (!isFromYou) return;
    const text = (msg.text || '').trim();
    if (!text) return;
    const target = msg.group || msg.handle;
    const senderId = `imessage_${msg.handle}${msg.group ? `_${msg.group}` : ''}`;
    try {
      const reply = await sendToAgent(senderId, text);
      await imessage.send(target, reply);
    } catch (e) {
      await imessage.send(target, `Error: ${e.message}`);
    }
  });
  console.log('iMessage bridge running — only your messages go to the agent (forwarding to %s)', AGENT_URL);
}

const args = process.argv.slice(2);
const telegramOnly = args.includes('--telegram');
const discordOnly = args.includes('--discord');
const imessageOnly = args.includes('--imessage');
const all = args.includes('--all');

if (telegramOnly) {
  runTelegram();
} else if (discordOnly) {
  runDiscord();
} else if (imessageOnly) {
  runImessage().catch((e) => console.warn('iMessage:', e.message));
} else if (all || (!telegramOnly && !discordOnly && !imessageOnly)) {
  runTelegram();
  runDiscord();
  runImessage().catch((e) => console.warn('iMessage:', e.message));
}

const anyEnabled = process.env.TELEGRAM_BOT_TOKEN || process.env.DISCORD_BOT_TOKEN || process.platform === 'darwin';
if (!anyEnabled || (telegramOnly && !process.env.TELEGRAM_BOT_TOKEN) || (discordOnly && !process.env.DISCORD_BOT_TOKEN)) {
  if (!process.env.TELEGRAM_BOT_TOKEN && !process.env.DISCORD_BOT_TOKEN) {
    console.log('Set TELEGRAM_BOT_TOKEN and/or DISCORD_BOT_TOKEN for those bots.');
  }
  console.log('Web UI: open %s in a browser.', AGENT_URL);
}
