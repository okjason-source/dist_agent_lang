(function () {
  var apiBase = window.location.origin;
  var messages = document.getElementById('messages');
  var form = document.getElementById('form');
  var input = document.getElementById('input');
  var aiDot = document.getElementById('ai-dot');
  var aiLabel = document.getElementById('ai-label');
  var aiFix = document.getElementById('ai-fix');
  var runningIndicator = document.getElementById('running-indicator');
  var frontendBadge = document.getElementById('frontend-badge');
  var checkInFlight = false;
  var runningCount = 0;

  function setRunning(inc) {
    runningCount = Math.max(0, runningCount + inc);
    if (runningIndicator) {
      if (runningCount > 0) {
        runningIndicator.textContent = 'Running…';
        runningIndicator.classList.add('visible');
      } else {
        runningIndicator.textContent = '';
        runningIndicator.classList.remove('visible');
      }
    }
  }

  function setStatus(connected, msg) {
    aiDot.className = 'ai-dot ' + (connected ? 'connected' : 'disconnected');
    aiLabel.textContent = connected ? 'LLM connected' : (msg || 'LLM disconnected');
    if (connected) {
      aiFix.classList.remove('visible');
    } else {
      aiFix.classList.add('visible');
      aiFix.innerHTML = '<strong>Setup:</strong> Set OPENAI_API_KEY in .env and restart with ./start.sh';
    }
  }

  async function checkAi() {
    if (checkInFlight) return;
    checkInFlight = true;
    if (aiDot.className.indexOf('connected') < 0) {
      aiDot.className = 'ai-dot checking';
      aiLabel.textContent = 'Checking…';
    }
    try {
      var r = await fetch(apiBase + '/api/ai-status');
      var d = await r.json().catch(function () { return {}; });
      setStatus(!!d.connected, d.message);
    } catch (e) {
      setStatus(false, 'Cannot reach server');
    }
    checkInFlight = false;
  }
  checkAi();
  setInterval(checkAi, 45000);

  (function initFrontendBadge() {
    if (!frontendBadge) return;
    fetch(apiBase + '/api/status')
      .then(function (r) { return r.json(); })
      .then(function (d) {
        var src = (d && d.frontend_source) || 'unknown';
        frontendBadge.textContent = src === 'public' ? 'Public UI' : (src === 'fallback' ? 'Fallback UI' : '?');
        frontendBadge.classList.add(src === 'public' ? 'public' : 'fallback');
      })
      .catch(function () {
        frontendBadge.textContent = '?';
        frontendBadge.classList.add('fallback');
      });
  })();

  function escapeHtml(s) {
    return String(s).replace(/&/g, '&amp;').replace(/</g, '&lt;').replace(/>/g, '&gt;').replace(/"/g, '&quot;');
  }

  function nl2br(s) {
    return String(s).replace(/\n/g, '<br>');
  }

  var toastTimeout = null;
  function showJobToast(success, message) {
    var toast = document.getElementById('job-toast');
    if (!toast) return;
    clearTimeout(toastTimeout);
    toast.textContent = message || (success ? 'Job done.' : 'Job failed.');
    toast.className = 'job-toast visible ' + (success ? 'success' : 'error');
    toastTimeout = setTimeout(function () {
      toast.classList.remove('visible');
    }, 4000);
  }

  function addMsg(who, text) {
    var d = document.createElement('div');
    d.className = 'msg ' + who;
    d.innerHTML = '<div class="who">' + who + '</div><div>' + nl2br(escapeHtml(text)) + '</div>';
    messages.appendChild(d);
    messages.scrollTop = messages.scrollHeight;
  }

  form.addEventListener('submit', async function (e) {
    e.preventDefault();
    var content = input.value.trim();
    if (!content) return;
    addMsg('user', content);
    input.value = '';
    input.focus();
    setRunning(1);
    try {
      var r = await fetch(apiBase + '/api/message', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ sender_id: 'web', content: content })
      });
      var d = await r.json().catch(function () { return {}; });
      var replyText = d.reply || d.error || (r.ok ? 'No reply' : 'Error ' + r.status);
      addMsg('agent', replyText);
      var ok = r.ok && !d.error;
      showJobToast(ok, ok ? 'Job done.' : 'Job failed: ' + (d.error || ('Error ' + r.status)));
      refreshHistory();
    } catch (err) {
      addMsg('agent', 'Error: ' + (err && err.message || String(err)));
      showJobToast(false, 'Job failed: ' + (err && err.message || String(err)));
    } finally {
      setRunning(-1);
    }
  });

  // --- History panel ---
  var historyList = document.getElementById('history-list');
  var historyRefresh = document.getElementById('history-refresh');
  var historyClear = document.getElementById('history-clear');

  function renderHistory(entries) {
    historyList.innerHTML = '';
    if (!entries || entries.length === 0) {
      historyList.innerHTML = '<div class="history-empty">No history yet. Chat or run a task to see entries here.</div>';
      return;
    }
    entries.forEach(function (e) {
      var el = document.createElement('div');
      el.className = 'history-item';
      var meta = (e.ts || '') + ' · ' + (e.type || 'message') + (e.sender ? ' · ' + escapeHtml(e.sender) : '');
      el.innerHTML =
        '<div class="meta">' + escapeHtml(meta) + '</div>' +
        '<div class="content">' + nl2br(escapeHtml((e.content || '').slice(0, 200))) + (e.content && e.content.length > 200 ? '…' : '') + '</div>' +
        (e.reply ? '<div class="reply">' + nl2br(escapeHtml((e.reply || '').slice(0, 150))) + (e.reply && e.reply.length > 150 ? '…' : '') + '</div>' : '');
      historyList.appendChild(el);
    });
  }

  async function refreshHistory() {
    try {
      var r = await fetch(apiBase + '/api/history');
      var d = await r.json().catch(function () { return {}; });
      if (d.ok && Array.isArray(d.entries)) renderHistory(d.entries);
      else renderHistory([]);
    } catch (e) {
      renderHistory([]);
    }
  }

  historyRefresh.addEventListener('click', refreshHistory);
  historyClear.addEventListener('click', async function () {
    if (!confirm('Clear all conversation history?')) return;
    try {
      await fetch(apiBase + '/api/history/clear', { method: 'POST' });
      refreshHistory();
    } catch (e) {}
  });

  // Load history on first show
  refreshHistory();

  // --- Task panel ---
  var taskForm = document.getElementById('task-form');
  var taskInput = document.getElementById('task-input');
  var taskResult = document.getElementById('task-result');

  taskForm.addEventListener('submit', async function (e) {
    e.preventDefault();
    var desc = taskInput.value.trim();
    if (!desc) return;
    taskResult.textContent = 'Running…';
    taskResult.className = 'panel-result loading';
    setRunning(1);
    try {
      var r = await fetch(apiBase + '/api/task', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ description: desc })
      });
      var d = await r.json().catch(function () { return {}; });
      taskResult.className = 'panel-result';
      if (d.error) {
        taskResult.textContent = d.error;
        taskResult.classList.add('error');
        showJobToast(false, 'Task failed: ' + d.error);
      } else {
        taskResult.textContent = d.result || 'Done.';
        taskResult.classList.remove('error');
        showJobToast(true, 'Job done.');
        refreshHistory();
      }
    } catch (err) {
      taskResult.className = 'panel-result error';
      taskResult.textContent = 'Error: ' + (err && err.message || String(err));
      showJobToast(false, 'Task failed: ' + (err && err.message || String(err)));
    } finally {
      setRunning(-1);
    }
  });

  // --- Workflow panel ---
  var workflowForm = document.getElementById('workflow-form');
  var workflowSelect = document.getElementById('workflow-select');
  var workflowInput = document.getElementById('workflow-input');
  var workflowResult = document.getElementById('workflow-result');

  async function loadWorkflows() {
    try {
      var r = await fetch(apiBase + '/api/workflow/list');
      var d = await r.json().catch(function () { return {}; });
      workflowSelect.innerHTML = '';
      if (d.ok && Array.isArray(d.workflows) && d.workflows.length > 0) {
        d.workflows.forEach(function (w) {
          var opt = document.createElement('option');
          opt.value = w.name;
          opt.textContent = w.name + (w.description ? ' — ' + w.description.slice(0, 40) : '');
          workflowSelect.appendChild(opt);
        });
      } else {
        var opt = document.createElement('option');
        opt.value = '';
        opt.textContent = 'No workflows';
        workflowSelect.appendChild(opt);
      }
    } catch (e) {
      workflowSelect.innerHTML = '<option value="">Failed to load</option>';
    }
  }

  workflowForm.addEventListener('submit', async function (e) {
    e.preventDefault();
    var name = workflowSelect.value;
    var inp = workflowInput.value.trim();
    if (!name || !inp) return;
    workflowResult.textContent = 'Running workflow…';
    workflowResult.className = 'panel-result loading';
    setRunning(1);
    try {
      var r = await fetch(apiBase + '/api/workflow', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({ workflow: name, input: inp })
      });
      var d = await r.json().catch(function () { return {}; });
      workflowResult.className = 'panel-result';
      if (d.error) {
        workflowResult.textContent = d.error;
        workflowResult.classList.add('error');
        showJobToast(false, 'Workflow failed: ' + d.error);
      } else {
        workflowResult.textContent = d.final_result || (d.ok ? 'Done.' : JSON.stringify(d));
        workflowResult.classList.remove('error');
        showJobToast(true, 'Job done.');
        refreshHistory();
      }
    } catch (err) {
      workflowResult.className = 'panel-result error';
      workflowResult.textContent = 'Error: ' + (err && err.message || String(err));
      showJobToast(false, 'Workflow failed: ' + (err && err.message || String(err)));
    } finally {
      setRunning(-1);
    }
  });

  loadWorkflows();

  // --- Bottom panel splitters ---
  function initPanelSplitters() {
    var container = document.getElementById('bottom-panels');
    if (!container) return;
    var left = document.getElementById('panel-history');
    var middle = document.getElementById('panel-task');
    var right = document.getElementById('panel-workflow');
    var firstDivider = container.querySelector('[data-divider="history-task"]');
    var secondDivider = container.querySelector('[data-divider="task-workflow"]');
    if (!left || !middle || !right || !firstDivider || !secondDivider) return;

    function apply(widthA, widthB, widthC) {
      left.style.flex = '0 0 ' + widthA + 'px';
      middle.style.flex = '0 0 ' + widthB + 'px';
      right.style.flex = '0 0 ' + widthC + 'px';
    }

    function current() {
      return {
        a: left.getBoundingClientRect().width,
        b: middle.getBoundingClientRect().width,
        c: right.getBoundingClientRect().width
      };
    }

    function drag(divider, side) {
      divider.addEventListener('pointerdown', function (ev) {
        if (window.matchMedia('(max-width: 768px)').matches) return;
        ev.preventDefault();
        var startX = ev.clientX;
        var minW = 220;
        var s = current();
        divider.classList.add('dragging');
        divider.setPointerCapture(ev.pointerId);

        function move(mev) {
          var delta = mev.clientX - startX;
          var a = s.a;
          var b = s.b;
          var c = s.c;
          if (side === 'left') {
            a = Math.max(minW, s.a + delta);
            b = Math.max(minW, s.b - delta);
          } else {
            b = Math.max(minW, s.b + delta);
            c = Math.max(minW, s.c - delta);
          }
          apply(a, b, c);
        }

        function up() {
          divider.classList.remove('dragging');
          window.removeEventListener('pointermove', move);
          window.removeEventListener('pointerup', up);
        }

        window.addEventListener('pointermove', move);
        window.addEventListener('pointerup', up);
      });
    }

    drag(firstDivider, 'left');
    drag(secondDivider, 'right');
  }

  initPanelSplitters();
})();
