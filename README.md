nix-switcher
============

A small helper for applying my NixOS and HomeManager configurations to my systems.

contribution
------------

As an experiment, this does not use the regular GH PR workflow, but gerrit
instead.

1. clone including the hook:
   ```sh
   git clone "ssh://NobbZ@gerrithub.io:29418/NobbZ/nix-switcher" && (cd "nix-switcher" && f=`git rev-parse --git-dir`/hooks/commit-msg ; mkdir -p $(dirname $f) ; curl -Lo $f https://review.gerrithub.io/tools/hooks/commit-msg ; chmod +x $f)
   ```
2. do your changes, try to keep them small, have them in a single commit!
3. Push them to gerrit:
   ```sh
   git push origin HEAD:refs/for/main
   ```
4. When iterating make sure to use `--amend` to update that one commit, do not create new commits!
