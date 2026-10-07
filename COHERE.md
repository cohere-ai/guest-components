# Cohere fork of guest-components

This fork carries a small set of Cohere patches on top of upstream
[confidential-containers/guest-components](https://github.com/confidential-containers/guest-components)
releases.

## Branches and tags

- `main`: mirror of upstream `main`. Never contains Cohere changes. Nothing
  syncs it automatically; repository admins update it by hand when needed:

  ```bash
  git fetch https://github.com/confidential-containers/guest-components.git main
  git push origin FETCH_HEAD:main
  ```

  A push to `main` runs upstream's own workflows, such as `publish-artifacts`
  and `coco-extension-image`, and publishes upstream builds (not Cohere ones)
  to `ghcr.io/cohere-ai/guest-components/*`. Nothing Cohere ships consumes
  those.
- `cohere-v<X.Y.Z>`: one branch per supported upstream release. It is the
  upstream `v<X.Y.Z>` tag plus the Cohere patches, with no merge commits.
  `git log v<X.Y.Z>..cohere-v<X.Y.Z>` lists every Cohere change.
- `v<X.Y.Z>-cohere.<N>`: Cohere releases built from `cohere-v<X.Y.Z>`.
- `cohere`: legacy branch from before version branches. Its last state is
  tagged `v0.20.0-cohere.1`.

Artifacts are published to `ghcr.io/cohere-ai/guest-components/*` on every
push to a `cohere-v*` branch, tagged with the commit SHA.
cloud-api-adaptor pins guest-components by that SHA in `versions.yaml`.

`publish-artifacts` can also be run manually (`workflow_dispatch`) from any
branch, for example to test an upgrade branch from cloud-api-adaptor before it
merges. Those artifacts land in the same GHCR repositories. For this repository,
cloud-api-adaptor's `hack/verify-provenance.sh` only accepts artifacts built from
`cohere` or `cohere-v<X.Y.Z>`, so a PodVM build rejects them unless it explicitly
allows that one branch with `PROVENANCE_EXTRA_REF` (dev builds only).

## Protection

The `cohere version branches` ruleset applies to `refs/heads/cohere-v*`:

- only the confidential-computing team can create a version branch
- changes need a pull request with one approving review
- no force-push, no deletion, linear history only

Repository admins can bypass the ruleset. That is what makes the
fast-forward in "Upgrading to a new upstream release" possible.

## Changing a version branch

Open a pull request into the current `cohere-v<X.Y.Z>` branch and merge it
with squash or rebase.

If the change fixes an existing patch, make it a fixup commit so it folds into
that patch at the next upgrade:

```bash
git commit --fixup=<sha-of-the-patch>
```

A new feature becomes its own patch.

## Upgrading to a new upstream release

Fetch the upstream release tag into the fork first, so the steps below can
use it:

```bash
git fetch https://github.com/confidential-containers/guest-components.git \
  tag v<NEW>
git push origin v<NEW>
```

1. Fold fixups into their patches:

   ```bash
   git switch -c upgrade/v<NEW> cohere-v<OLD>
   git rebase -i --autosquash v<OLD>
   ```

2. Replay the patches onto the new release:

   ```bash
   git rebase --onto v<NEW> v<OLD>
   ```

   Git stops at each patch that conflicts. Resolve it, `git add`, and
   `git rebase --continue`. Patches upstream already contains drop out.

3. Create `cohere-v<NEW>` from the `v<NEW>` tag (confidential-computing team),
   then open a pull request from `upgrade/v<NEW>` into it. Include the
   range-diff in the description:

   ```bash
   git range-diff v<OLD>..cohere-v<OLD> v<NEW>..upgrade/v<NEW>
   ```

4. After approval and testing, fast-forward the branch to the reviewed commits
   so the published SHAs match. Don't use GitHub's merge button: rebase
   rewrites every SHA, and squash folds the patches into one commit.

   ```bash
   git push origin upgrade/v<NEW>:cohere-v<NEW>
   ```

   The ruleset requires a pull request, so only repository admins can make
   this push; it is recorded as a ruleset bypass. GitHub then marks the pull
   request as merged.

5. Tag the release `v<NEW>-cohere.1` and pin its SHA in cloud-api-adaptor.

The previous version branch stays as it is, so artifacts built from it remain
reachable.
