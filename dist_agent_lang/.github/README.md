# GitHub Configuration

This directory contains GitHub-specific configuration files for the dist_agent_lang repository.

## 📁 Directory Structure

```
.github/
├── ISSUE_TEMPLATE/          # Issue templates
│   ├── bug_report.md       # Template for bug reports
│   ├── feature_request.md  # Template for feature requests
│   ├── good_first_issue.md # Template for good first issues
│   ├── question.md         # Template for questions
│   └── config.yml          # Issue template configuration
├── workflows/              # GitHub Actions workflows
│   ├── ci.yml             # Continuous Integration
│   └── release.yml        # Release automation
├── scripts/               # Utility scripts
│   └── import_labels.sh   # Script to import GitHub labels
├── labels.json            # Label definitions (reference)
├── pull_request_template.md # PR template
├── SETUP.md               # Setup instructions
└── README.md              # This file
```

## 🚀 Quick Start

### For Repository Maintainers

1. **Enable GitHub Discussions**
   - Go to Settings → General → Features
   - Enable "Discussions"

2. **Import Labels**
   ```bash
   cd .github/scripts
   ./import_labels.sh
   ```

3. **Verify Templates**
   - Create a test issue to verify templates work
   - Create a test PR to verify PR template works

See [SETUP.md](SETUP.md) for detailed instructions.

### For Contributors

- **Reporting Bugs**: Use the "Bug Report" template when creating an issue
- **Requesting Features**: Use the "Feature Request" template
- **Asking Questions**: Use the "Question" template (or GitHub Discussions)
- **Submitting PRs**: The PR template will appear automatically

## 📋 Files Overview

### Issue Templates

- **`bug_report.md`**: Structured template for bug reports
- **`feature_request.md`**: Template for feature suggestions
- **`good_first_issue.md`**: Template for beginner-friendly tasks
- **`question.md`**: Template for questions
- **`config.yml`**: Configures the issue template chooser

### Pull Request Template

- **`pull_request_template.md`**: Comprehensive PR template with checklists

### Labels

- **`labels.json`**: Reference file with all label definitions
- **`scripts/import_labels.sh`**: Script to import labels to GitHub

### Documentation

- **`SETUP.md`**: Step-by-step setup guide
- **`README.md`**: This file

## 🔧 Maintenance

### Adding New Labels

1. Add to `.github/labels.json`
2. Update `.github/scripts/import_labels.sh`
3. Run the import script

### Updating Templates

1. Edit the template files in `.github/ISSUE_TEMPLATE/`
2. Test by creating a test issue
3. Commit and push changes

### Updating PR Template

1. Edit `.github/pull_request_template.md`
2. Test by creating a test PR
3. Commit and push changes

## 📚 Related Documentation

- [CONTRIBUTING.md](../CONTRIBUTING.md) - Main contribution guide
- [GOOD_FIRST_ISSUES.md](../GOOD_FIRST_ISSUES.md) - Beginner-friendly tasks
- [TESTING_GUIDE.md](../TESTING_GUIDE.md) - Testing guide
- [CONTRIBUTOR_STRATEGY.md](../CONTRIBUTOR_STRATEGY.md) - Community strategy

## 🆘 Troubleshooting

### Templates Not Showing

- Ensure files are committed and pushed
- Check file names match exactly
- Verify files are in correct directories

### Labels Not Importing

- Ensure GitHub CLI is installed and authenticated
- Check repository permissions
- Try manual creation as fallback

See [SETUP.md](SETUP.md) for detailed troubleshooting.

---

**Note**: These files are part of the repository and should be committed to version control.
