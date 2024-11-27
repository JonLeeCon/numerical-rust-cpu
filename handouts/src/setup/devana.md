# Devana cluster

## Introduction

Because this training is co-organized by the [Slovak National Center of
Competence for HPC](https://eurocc.nscc.sk/en/), some participants could be
granted to the [Devana computing
cluster](https://userdocs.nscc.sk/devana/system_overview/introduction/). This
allows them to get access to a pre-configured environment with abundant compute
resources.

If you are one of these participants, please read through the following
instructions. Otherwise, you will need to use one of the [other environment
setup methods](pathfinding.md).


## Enabling SSH login

I advise you to register your SSH key on the Devana login nodes for two reasons:

1. You will be able to access the cluster using your usual terminal, which may
   feel more familiar and easier to use than the web interface.
2. More importantly, you will then be able to leverage excellent SSH-based local
   tools to manipulate files on the cluster as if they were resident on your
   local machine. This will notably let you use your usual code editor,
   without being bothered by network lag.

Because password-based SSH login is not enabled on Devana, you cannot do this
key registration using SSH itself or ssh-based tools like `ssh-copy-id`. You
will need to use the [cluster's web interface](https://ood.devana.nscc.sk/) to
manually add your public key to the `~/.ssh/authorized_keys` file on your Devana
account.

<details>

<summary>If you are unfamiliar with this process, please click here for more details</summary>

Your public SSH key is the content of a text file that is typically stored with
extension `.pub` inside of folder `~/.ssh` of your local machine. Common names
include `~/.ssh/id_ed25519.pub` or `~/.ssh/id_rsa.pub`. If you have several of
them, you can likely pick any of them: unless you used a particularly exotic key
format (in which case you are unlikely to be reading these lines) any of your
keys will most likely work fine for our purposes.

From a local terminal on your computer, you can enumerate available public keys
files with the commands `ls ~/.ssh/*.pub` and display the contents of one of
them using the `cat` command (e.g. `cat ~/.ssh/id_ed25519.pub`)[^1]. You should
get output that looks like this:

```text
ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOCK1IUuQ5WuXotZUhEURBYkslMSUlH667CAwFJAMUIV grasland@lal.in2p3.fr
```

If you do not find any public key file or get an error message stating that the
`~/.ssh` directory does not exist, it likely means that you have never generated
an SSH key for the machine that you're using. The SSH company provides a handy
guide in the ["Creating an SSH Key Pair for User Authentication" section of this
page](https://www.ssh.com/academy/ssh/keygen#creating-an-ssh-key-pair-for-user-authentication)
that should work on Linux, macOS and Windows >= 10. After this is done, the
above instructions should work.

With your public key on sight, you can copy it, then add it to your Devana
command by running a command similar to the following inside of a [Devana
shell](https://ood.devana.nscc.sk/pun/sys/shell/ssh/login02):

```bash
# The following command adds my SSH public key, please modify it to add yours
echo "ssh-ed25519 AAAAC3NzaC1lZDI1NTE5AAAAIOCK1IUuQ5WuXotZUhEURBYkslMSUlH667CAwFJAMUIV grasland@lal.in2p3.fr"  \
     >> ~/.ssh/authorized_keys
```

Notice the use of the **two** output redirection "arrows". This is necessary in order not to override existing SSH keys that were automatically set up by the Devana user management system.

Should you get stuck when trying to perform this procedure, I advise [opening an
NSCC support ticket](https://support.nscc.sk/open.php) for assistance, but I can
also attempt to provide live support myself during the course.
      
[^1]: Thanks to [PowerShell compatibility
      aliases](https://learn.microsoft.com/en-us/powershell/scripting/learn/shell/using-aliases?view=powershell-7.4#compatibility-aliases-in-windows),
      these Unix commands that work on Linux and macOS should also work on
      modern versions of Windows.

</details>


## Improving SSH ergonomics

You should now be able to log in to Devana over SSH using the following
parameters:

- Host: login.devana.nscc.sk
- Port: 5522
- User: (your Devana username)

On systems that use the OpenSSH client, like Linux and macOS, you can avoid
repeating this configuration on every login by creating an entry like this in
your `~/.ssh/config` file:

```text
Host devana
    User your_devana_username
    HostName login.devana.nscc.sk
    Port 5522
```

This way, you can just type in `ssh devana` and the right parameters will be
picked automatically. And this will also work with SSH-based tools like `scp` and `sshfs`.

On Windows, you will likely be using a graphical SSH client, in which case a
similar result can be achieved by letting the tool memorize your last login
configuration(s).

But this will not address the issue that editing files over SSH is a bit of a
pain due to limited text editor choice and network lag. Here are some extra
tooling suggestions that will make this process more comfortable by letting you
edit files on Devana using the local text editor that you are used to, without
feeling the network lag in the process:

- If you are an everyday VSCode user, that editor's
  [Remote-SSH](https://code.visualstudio.com/docs/remote/ssh) plugin is worth
  checking out. It will let you edit your Devana files with VSCode almost as if
  they were inside of a local directory on your machine, and the editor's
  integrated terminal will automatically open SSH shells on the cluster instead
  of opening local shells on your machine.
- If you prefer another editor and use Linux, macOS, or another Unix system, you
  can get a similar file editing experience by using
  [sshfs](https://github.com/libfuse/sshfs) to mount your Devana home directory
  into your host system, then use your text editor to open the resulting mount
  point. Combined with an SSH shell into the cluster, this should also provide
  pretty good ergonomics.
- If you use Windows, similar ergonomics can be achieved by using the
  [MobaXTerm](https://mobaxterm.mobatek.net/) SSH client, whose file management 
  pane lets you easily open and locally edit files stored on the cluster.

In the remainder of this chapter, we will assume that all this has all been
taken care of. Each command in the remainder of this chapter must be typed
inside of a Devana shell.


## Environment setup

Like most HPC centers, Devana uses [environment
modules](https://en.wikipedia.org/wiki/Environment_Modules_(software)) to allow
multiple incompatible versions of applications and libraries to be installed.

With this approach to dependency management, your Devana shell initially "sees"
almost none of the HPC software that's installed on the cluster. That software
must be "brought in scope" using `module load` commands. For this course, the
command you want is:

```bash
module load hwloc/2.9.2-GCCcore-13.2.0  \
            HDF5/1.14.0-iimpi-2023a  \
            Rust-nightly/2024-10-28
```

Alas, you must type this command for every shell you open on Devana before you
can do Rust work, which can get boring quickly if you're the kind of person who
opens lots of short-lived shells.

If you're not using Devana for any other purpose than doing this Rust course, it
may therefore be more convenient to just memorize this environment configuration
and have it automatically applied on every subsequent Devana login with the
following command:

```bash
echo "module load hwloc/2.9.2-GCCcore-13.2.0 HDF5/1.14.0-iimpi-2023a Rust-nightly/2024-10-28"  \
     >> ~/.profile
```

But at the end of the course, if you still have access to Devana for other
purposes, you will likely want to delete the associated line in your
`~/.profile` file, so you can safely load other environment modules without any
risk of conflict or unexpected interaction.


## Slurm policy and basics

On Devana, building programs on login nodes is tolerated, but any work that's
more compute-intensive (benchmarks, simulation) should be offloaded to the
cluster's dedicated worker nodes. This is done by using the [Slurm workload
manager](https://slurm.schedmd.com/documentation.html).

Slurm is very sophisticated and powerful, so we will not attempt to explain all about it in this chapter. Instead, we will just give you the 5-minutes getting started tour and a few basic commands. For more information, please check out the [NSCC-provided documentation](https://userdocs.nscc.sk/devana/job_submission/slurm_quick_start/).

---

The easiest way to run a command on a worker node using Slurm is to type in the
`srun` command, followed by the command that you want to run on the worker node.
For example, this command...

```bash
srun echo "Hello from a worker node"
```

...will yield the following result:

```text
srun: slurm_job_submit: Job's time limit was set to partition limit of 1440 minutes.
srun: job 574183 queued and waiting for resources
srun: job 574183 has been allocated resources
Hello from a worker node
```

After a few lines of srun reminding your default job duration limit and tracking the process of allocating worker ressources, the expected `stdout` output from `echo` will appear at the end.

---

But if you try to run tooling with a more sophisticated Textual User Interface
(TUI) like Rust's `cargo` build system this way, you will quickly notice that it
behaves a little differently than usual: all text colors are gone, dynamic
displays like progress bars are replaced with simplified versions, and keyboard
input is not processed normally.

In fact, software that's mainly built around keyboard input like the `vi` text
editor will not work well at all: its display will be visually corrupt, with text input writing over the text output region.

This happens because by default `srun` does not allocate a pseudo-terminal
(PTY), as that requires setting up a slightly more sophisticated communication
infrastructure between the login node and worker node for the duration of the
job. But you can override this default with the `--pty` option...

```text
srun --pty ... your command goes here ...
```

...and then even sophisticated TUI apps will work as if you ran them on the
login node.

---

On parallel CPU work like compilation, however, you will soon notice that
execution via the above basic `srun` configurations is much slower than direct
execution on the login node. Closer examination will reveal that they are using
only one CPU core, which may rightfully concern you.

This is actually just Slurm allocating a single CPU core to your job by default,
so that you use as little resources as possible when you've not expressed a need
for more. You can ask for `N` CPU cores instead by simply specifying the `-cN`
option to `srun`. For example `srun -c3 some-command` allocates 3 CPU cores to
the execution of the command `some-command`.

Knowing that...

- This particular `srun` option only lets you allocate CPU cores on a single
  worker node[^2]
- Devana's worker nodes have 2 NUMA nodes with 32 cores each, hence...
  1. Requests for >64 cores like `-c65` are meaningless and cannot be fulfilled
     by Slurm.
  2. Requests for >32 cores like `-c33` are unlikely to yield significant
     performance benefits over `-c32`, and in fact will likely run _slower_ due
     to NUMA effects, unless the software you're running has received special
     tuning to leverage NUMA correctly.
- The more cores you ask for, the longer you will wait for resources to free up
  on the cluster

...you should get a good intuition of how many CPU cores you should ask for,
depending on what your job is trying to do.

---

Overall, `srun --pty -cN ... your command ...` is pretty much all the Slurm
vocabulary you'll need to know for this course. But again, if you want to know
more about Slurm's more advanced possibilities like job scripts that let you avoid typing the same options over and over again, feel free to check out [the local Slurm documentation](https://userdocs.nscc.sk/devana/job_submission/slurm_quick_start/).

[^2]: There are ways to request CPU cores spread over multiple worker nodes
      using options like `-B Sockets:Cores:Threads`. But we will not cover
      distributed computing in this course, and will thus not be able to
      leverage these distributed allocations.
