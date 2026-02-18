# CI/CD Troubleshooting (fresh deployment)

If workflows **don’t run at all** after a fresh push to a new/empty repo, check the following.

## 1. Actions are enabled

- Repo **Settings** → **Actions** → **General**
- Under “Actions permissions”, choose **“Allow all actions and reusable workflows”** (or at least the ones you use)
- If “Disable actions” is selected, no workflow will run

## 2. Default branch matches workflow triggers

- Workflows are set to run on **`main`** (and **`develop`** for CI only)
- Repo **Settings** → **General** → **Default branch** should be **`main`**
- If the default branch is something else (e.g. `master`), either:
  - Rename it to `main`, or
  - Change the workflow `on.push.branches` / `on.pull_request.branches` to match

## 3. `.github` was actually pushed

- Your `scripts/push_to_github.sh` does **not** ignore `.github/`, so it should be in the repo
- On GitHub, open the repo → **Code** and confirm these exist:
  - `.github/workflows/ci.yml`
  - `.github/workflows/security.yml`
  - `.github/workflows/codeql-analysis.yml`
  - `.github/workflows/release.yml`
- If they’re missing, push again (or fix the script) so `.github` is included

## 4. First push went to the right branch

- Push to **`main`** (e.g. `git push -u origin main`) so **push**-triggered workflows run
- If you only pushed to another branch, either push/merge to `main` or add that branch to `on.push.branches` in the workflows

## 5. Run workflows manually (sanity check)

- **Actions** tab → select **“CI/CD Pipeline”** or **“Security Checks”**
- Click **“Run workflow”** (only if the workflow has `workflow_dispatch` in `on:`)
- If the run appears and starts, triggers and permissions are fine; any failure is in the job steps (e.g. Cargo, tests)

## 6. Token permissions (if jobs fail with “permission denied”)

- Repo **Settings** → **Actions** → **General** → **Workflow permissions**
- Use **“Read and write permissions”** if the workflow needs to write (e.g. upload artifacts, deploy)

---

**Summary:** Most often “CI is off” after a fresh deploy is because **Actions are disabled** or the **default branch** isn’t `main`. Enable Actions and set default branch to `main` (or adjust workflow branches), then push again or run the workflow manually.
