<div align="center">

<img src="./assets/com.spydr06.logicrs.Devel.png" width="200"/>

<h1>Contributing to LogicRs</h1>

</div>

Contributions to LogicRs are very welcome. For major changes, please open an issue first to discuss what you would like to change.

Following are instructions, hints and guidelines how contributions should be worked on.

## Code Structure

The code is structures into three main folders:
- **`content`** contains XML-style `.ui`-files which lay out key components of the GTK UI frontend.
- **`src`** contains all rust source code for the application.
- **`style`** contains css-files used for the styling of the application.
  
Building LogicRs is done via Cargo. On UNIX, dependencies should be managed either directly by Cargo or your distribution's package manager. 
On Windows, a special build environment like MSYS64/MINGW64 is required.

Successfully built executables will be put into the **`target/`** directory.

## Contribution Workflow

This is an example workflow showing how contributions can be done.
Since git is very flexible, there are many ways how this can be achieved.

> (If you don't already have a GitHub account, please create one. Your GitHub username will be referred to later as 'YOUR_GITHUB_USERNAME'. Change it accordingly in the steps below.)

1. Fork https://github.com/spydr06/logicrs using GitHub's interface to your own account. Let's say that the forked repository is at
`https://github.com/YOUR_GITHUB_USERNAME/logicrs`.

2. Clone the main LogcRs repository https://github.com/spydr06/logicrs to a local folder on your computer, say named `logicrs-dev/` (`git clone https://github.com/spydr06/logicrs logicrs-dev`)
3. `cd logicrs-dev`
4. `git remote add pullrequest https://github.com/YOUR_GITHUB_USERNAME/logicrs`
> the remote named `pullrequest` should point to YOUR own forked repo, **not the main CSpydr repository**! 
After this, your local cloned repository is prepared for making pullrequests, and you can just do normal git operations such as:
`git pull` `git status` and so on.

5. When finished with a feature/bugfix/change, you can:
`git checkout -b fix_<your thing>`
   - Don't forget to keep formatting standards before committing
6. `git push pullrequest`  # (NOTE: the `pullrequest` remote was setup on step 4)
7. On GitHub's web interface, go to: https://github.com/spydr06/logicrs/pulls

   Here the UI shows a dialog with a button to make a new pull request based on the new pushed branch.

8. After making your pullrequest (aka, PR), you can continue to work on the branch `fix_<your thing>` ... just do again `git push pullrequest` when you have more commits.

9. If there are merge conflicts, or a branch lags too much behind V's main, you can do the following:

   1. `git pull --rebase origin main` # solve conflicts and do
   `git rebase --continue`
   2. `git push pullrequest -f` # this will overwrite your current remote branch
   with the updated version of your changes.

The point of doing the above steps, is to never directly push to the main LogicRs repository, *only to your own fork*. Since your local `main` branch tracks the
main LogicRs repository's main, then `git checkout main`, as well as
`git pull --rebase origin main` will continue to work as expected


