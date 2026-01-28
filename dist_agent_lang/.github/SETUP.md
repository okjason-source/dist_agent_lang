# GitHub Repository Setup Guide

This guide explains how to set up GitHub features for the dist_agent_lang repository.

## ✅ What's Already Set Up

The following files are already in place:

- ✅ **Issue Templates** (`.github/ISSUE_TEMPLATE/`)
  - `bug_report.md` - Template for bug reports
  - `feature_request.md` - Template for feature requests
  - `good_first_issue.md` - Template for good first issues
  - `question.md` - Template for questions
  - `config.yml` - Configuration for issue templates

- ✅ **Pull Request Template** (`.github/pull_request_template.md`)
  - Comprehensive PR template with checklists

- ✅ **Labels Configuration** (`.github/labels.json`)
  - JSON file with all recommended labels

- ✅ **Contributors File** (`CONTRIBUTORS.md`)
  - File to recognize contributors

## 🚀 Manual Setup Steps

### 1. Enable GitHub Discussions

**Steps:**
1. Go to your GitHub repository
2. Click **Settings** → **General**
3. Scroll down to **Features** section
4. Check **Discussions**
5. Click **Set up discussions**

**Create Categories:**
- **Q&A** - Questions and answers
- **Ideas** - Feature suggestions and discussions
- **Showcase** - Share what you've built with dist_agent_lang
- **General** - General discussion

**Time**: ~5 minutes

---

### 2. Import Labels

GitHub doesn't automatically import labels from JSON, but you can use the GitHub CLI or manually create them.

#### Option A: Using GitHub CLI (Recommended)

```bash
# Install GitHub CLI if you haven't: https://cli.github.com/

# Authenticate
gh auth login

# Import labels
gh api repos/:owner/:repo/labels -f name="good-first-issue" -f color="0e8a16" -f description="Good for newcomers"
gh api repos/:owner/:repo/labels -f name="help-wanted" -f color="008672" -f description="Extra attention is needed"
gh api repos/:owner/:repo/labels -f name="bug" -f color="d73a4a" -f description="Something isn't working"
gh api repos/:owner/:repo/labels -f name="enhancement" -f color="a2eeef" -f description="New feature or request"
gh api repos/:owner/:repo/labels -f name="documentation" -f color="0075ca" -f description="Improvements or additions to documentation"
gh api repos/:owner/:repo/labels -f name="testing" -f color="f9d71c" -f description="Testing related"
gh api repos/:owner/:repo/labels -f name="security" -f color="ee0701" -f description="Security issue"
gh api repos/:owner/:repo/labels -f name="question" -f color="d876e3" -f description="Further information is requested"
```

Or use a script to import all labels from `labels.json`:

```bash
# Create a script to import all labels
cat > import_labels.sh << 'EOF'
#!/bin/bash
# Import labels from labels.json

gh api repos/:owner/:repo/labels -f name="good-first-issue" -f color="0e8a16" -f description="Good for newcomers"
gh api repos/:owner/:repo/labels -f name="help-wanted" -f color="008672" -f description="Extra attention is needed"
gh api repos/:owner/:repo/labels -f name="bug" -f color="d73a4a" -f description="Something isn't working"
gh api repos/:owner/:repo/labels -f name="enhancement" -f color="a2eeef" -f description="New feature or request"
gh api repos/:owner/:repo/labels -f name="documentation" -f color="0075ca" -f description="Improvements or additions to documentation"
gh api repos/:owner/:repo/labels -f name="testing" -f color="f9d71c" -f description="Testing related"
gh api repos/:owner/:repo/labels -f name="security" -f color="ee0701" -f description="Security issue"
gh api repos/:owner/:repo/labels -f name="question" -f color="d876e3" -f description="Further information is requested"
gh api repos/:owner/:repo/labels -f name="wontfix" -f color="ffffff" -f description="This will not be worked on"
gh api repos/:owner/:repo/labels -f name="duplicate" -f color="cfd3d7" -f description="This issue or pull request already exists"
gh api repos/:owner/:repo/labels -f name="invalid" -f color="e4e669" -f description="This doesn't seem right"
gh api repos/:owner/:repo/labels -f name="blocked" -f color="b60205" -f description="Blocked by another issue"
gh api repos/:owner/:repo/labels -f name="priority: high" -f color="b60205" -f description="High priority issue"
gh api repos/:owner/:repo/labels -f name="priority: medium" -f color="fbca04" -f description="Medium priority issue"
gh api repos/:owner/:repo/labels -f name="priority: low" -f color="0e8a16" -f description="Low priority issue"
gh api repos/:owner/:repo/labels -f name="area: lexer" -f color="1d76db" -f description="Related to lexer/tokenization"
gh api repos/:owner/:repo/labels -f name="area: parser" -f color="1d76db" -f description="Related to parser/AST"
gh api repos/:owner/:repo/labels -f name="area: runtime" -f color="1d76db" -f description="Related to runtime execution"
gh api repos/:owner/:repo/labels -f name="area: stdlib" -f color="1d76db" -f description="Related to standard library"
gh api repos/:owner/:repo/labels -f name="area: blockchain" -f color="1d76db" -f description="Related to blockchain features"
gh api repos/:owner/:repo/labels -f name="area: ai" -f color="1d76db" -f description="Related to AI agent features"
gh api repos/:owner/:repo/labels -f name="area: security" -f color="1d76db" -f description="Related to security features"
gh api repos/:owner/:repo/labels -f name="area: http" -f color="1d76db" -f description="Related to HTTP server"
gh api repos/:owner/:repo/labels -f name="area: ffi" -f color="1d76db" -f description="Related to FFI (Foreign Function Interface)"
EOF

chmod +x import_labels.sh
./import_labels.sh
```

#### Option B: Manual Creation

1. Go to your GitHub repository
2. Click **Issues** → **Labels**
3. Click **New label**
4. Create each label manually using the information from `labels.json`

**Time**: ~10-15 minutes (manual) or ~2 minutes (with script)

---

### 3. Verify Issue Templates

**Steps:**
1. Go to your GitHub repository
2. Click **Issues** → **New issue**
3. You should see templates for:
   - Bug Report
   - Feature Request
   - Good First Issue
   - Question

**Time**: ~2 minutes

---

### 4. Verify PR Template

**Steps:**
1. Create a test branch
2. Make a small change
3. Open a Pull Request
4. Verify the template appears automatically

**Time**: ~5 minutes

---

### 5. Label Existing Issues

**Steps:**
1. Go to **Issues**
2. Review existing issues
3. Add appropriate labels:
   - `bug` for bugs
   - `enhancement` for features
   - `good-first-issue` for beginner-friendly tasks
   - `help-wanted` for tasks needing help
   - Area labels (`area: lexer`, `area: parser`, etc.)

**Time**: ~30 minutes (depending on number of issues)

---

## 📋 Setup Checklist

- [ ] Enable GitHub Discussions
- [ ] Create Discussion categories (Q&A, Ideas, Showcase, General)
- [ ] Import/create GitHub labels
- [ ] Verify issue templates work
- [ ] Verify PR template works
- [ ] Label existing issues appropriately
- [ ] Create 5-10 "good first issue" issues from `GOOD_FIRST_ISSUES.md`

---

## 🎯 Next Steps After Setup

1. **Create Initial Good First Issues**
   - Review `GOOD_FIRST_ISSUES.md`
   - Create GitHub issues for the easiest tasks
   - Label them as `good-first-issue`

2. **Update README**
   - Already done! The README links to all the guides.

3. **Share the Project**
   - Post on social media
   - Share in relevant communities
   - Highlight that contributors are welcome

4. **Engage with Contributors**
   - Respond to issues promptly
   - Review PRs quickly
   - Thank contributors publicly

---

## 🔧 Troubleshooting

### Issue Templates Not Showing

- Make sure files are in `.github/ISSUE_TEMPLATE/` directory
- Check that `config.yml` exists
- Verify file names end in `.md`
- Ensure files are committed and pushed

### PR Template Not Showing

- Make sure file is named `pull_request_template.md` (not `.md.txt`)
- Verify it's in `.github/` directory
- Ensure file is committed and pushed

### Labels Not Importing

- Use GitHub CLI method (Option A) for easiest import
- Or create labels manually (Option B)
- Labels must be created before they can be used

---

## 📚 Additional Resources

- [GitHub Issue Templates Guide](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/configuring-issue-templates-for-your-repository)
- [GitHub Pull Request Templates](https://docs.github.com/en/communities/using-templates-to-encourage-useful-issues-and-pull-requests/about-issue-and-pull-request-templates)
- [GitHub Discussions Guide](https://docs.github.com/en/discussions)
- [GitHub Labels Guide](https://docs.github.com/en/issues/using-labels-and-milestones-to-track-work/managing-labels)

---

**Once setup is complete, you're ready to welcome contributors!** 🎉
