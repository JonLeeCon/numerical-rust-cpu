# Using `rust_code_server`

The `rust_code_server` environment provides a pre-configured development
environment based on the VS Code editor, accessible from your web browser. It is
the recommended route if you want to get a working environment with minimal
effort on a native Linux system.

## Setting up Docker or Podman

You should first install one of [Docker
Engine](https://docs.docker.com/engine/install/) or
[Podman](https://podman.io/docs/installation#installing-on-linux), if you have
not done so already. The name of each container runtime in the previous sentence
links to the recommended setup instructions.

If you are unfamiliar with containers and just want the easiest default choice,
Docker is easiest to install and would therefore be my recommendation for
beginners. Just make sure that you do go for a native installation of Docker
Engine, and not a Docker Desktop installation. The latter is not suitable for
using `rust_code_server` because it uses a hidden virtual machine under the
hood, which will prevent you from using the integrated code editor later in this
tutorial.[^1]

After installation, you may need to do a little bit of extra system
configuration to make the container runtime usable by non-root users (as
explained in the installation instructions linked above). Then you should be
able to run a test container that prints a hello world message.

Please pick your container runtime below to see the appropriate command for this
test:

{{#tabs global="container-runtime"}}
{{#tab name="Docker"}}
```bash
docker run hello-world
```
{{#endtab}}
{{#tab name="Podman"}}
```bash
podman run hello-world
```
{{#endtab}}
{{#endtabs}}

[^1]: Without getting into details, we had to deal with a security/usability
      tradeoff here. Either we allowed everyone on your local network to access
      your code editor, or we broke Docker Desktop's hidden VM. We chose the
      secure option that breaks Docker Desktop but keeps your code editor
      private.


## Downloading the exercises source code

You may have been directed to this documentation some time ahead of the course,
at a point where the material may not be fully finalized yet. To handle material
evolution, and to allow you to save your work, we distribute the exercises'
source code via a [separate archive](exercises.zip) that you should unpack
somewhere on your machine. The resulting directory will then be mounted inside
of the container.

Provided that the `wget` and `unzip` utilities are installed, you can download
and unpack the source code in the current directory using the following sequence
of commands:

```bash
if [ -e exercises ]; then
    echo "ERROR: Please move or delete the existing 'exercises' subdirectory"
else
    wget https://numerical-rust-cpu-d1379d.pages.math.cnrs.fr/setup/exercises.zip  \
    && unzip exercises.zip  \
    && rm exercises.zip
fi
```

The following steps will assume that you are in the directory where the archive
has been extracted, as if you just ran the sequence of commands above (an
`exercises/` subdirectory should be present in the output of `ls`).


## Starting the container

Now that you have a working container runtime and the exercises' source code,
you should be able to run a container based on the development environment. We
will need to change a few things with respect to the default configuration of
`docker run` and `podman run`:

- For interactive use, we will want to follow the container's standard output
  and be able to interact with it using our terminal. This can be done using
  the `-it` pair of options.
- There is no point in keeping around interactive containers after we have
  stopped them, so it is a good idea to automatically delete the container after
  use via the `--rm` option.
- Our code editing environement uses an HTTP server on port 8080, which we must
  expose to be able to connect to it. The easiest way to do this is to use the
  `--net=host` option.
- And finally, we need to mount the exercises material into the container so
  that it can be used inside. This can be done using the `-v
  "$(pwd)/exercises":/home/jovyan/exercises` option.
    * If you use podman, then you should add an `:U` suffix at the end of this
      option so that non-privileged users inside of the container get write
      access to the code.

Putting this all together, we get the following Docker and Podman command lines:

{{#tabs global="container-runtime"}}
{{#tab name="Docker"}}
```bash
docker run --net=host --rm -it  \
           -v "$(pwd)/exercises":/home/jovyan/exercises  \
           registry.plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/rust_code_server:latest
```
{{#endtab}}
{{#tab name="Podman"}}
```bash
podman run --net=host --rm -it  \
           -v "$(pwd)/exercises":/home/jovyan/exercises:U  \
           registry.plmlab.math.cnrs.fr/grasland/numerical-rust-cpu/rust_code_server:latest
```
{{#endtab}}
{{#endtabs}}

If you get an error message about port 8080 already being in use, it likely
means that another software on your machine (e.g. Jupyter) is already listening
for network connections on port 8080. In that case, you must hunt and close the
offending software using tools like `ss -tnlp`.

Once you solve these TCP port allocation issues, you will get a few lines of
output notifying you that code-server is ready to serve HTTP requests. Before
that, there will be a message along these lines...

```text
### Use the following password: xxxxxxxxxxxxxxxxxxxxxxxx
```

...where xxxxxxxxxxxxxxxxxxxxxxxx is a string of hexadecimal digits. Copy this
string into your clipboard, then point your web browser to
the <http://127.0.0.1:8080> URL, and paste the password when asked to. Once that is
done, you should land into a browser-based version of VSCode.

{{#tabs global="container-runtime"}}
{{#tab name="Docker"}}

Sadly, as you will quickly figure out, this code editor is not fully functional
yet because it cannot write into the `~/exercises` directory that we have
mounted inside of the container.

To fix this, you will need to check back in the terminal where you ran the
`docker run` command, and look for the container's output right after the "Use
the following password" line:

```text
### If using Docker, mounted files must be chown'd to uuuu:gggg
```

...where `uuuu` and `gggg` are the UID and the GID of the user running the code
editor inside of the container. This is the information that we are going to
need next.

{{#endtab}}
{{#tab name="Podman"}}

At this point, you are mostly done: the code editor can be used immediately.

However, after closing the container, you will find that the file permissions of
the `exercises/` directory have become a little weird, for example you cannot
edit or delete files without sudo anymore. We will explain why this happens and
how it can be fixed in the next chapter.

{{#endtab}}
{{#endtabs}}


## Fixing up file permissions

One drawback of using Linux containers is that the users inside of the
containers do not match those of your local system, which causes all sorts of
access permission problems when sharing files between the container and the
host. Docker and Podman handle this issue a little differently.


{{#tabs global="container-runtime"}}
{{#tab name="Docker"}}

With Docker, you are responsible for changing the permissions of the directories
that you mount inside of the container, so that the user inside of the container
can access it.

This can be done by running the command `sudo chown -R uuuu:gggg exercises`,
where `uuuu` and `gggg` are the user and group ID that the container prints on
startup.

You must do this in order to let the provided code editor edit files in the
`exercises/` directory.

{{#endtab}}
{{#tab name="Podman"}}

Podman can change the file permissions of mounted directories automatically on
container startup, if you use the recommended `:U` option when mounting the host
directory inside of the container.

However, it will not restore the original file permissions when the container is
stopped, so your files will remain in a weird state where you don't have
permission to edit or delete them.

{{#endtab}}
{{#endtabs}}

At the end of the course, with both Docker and Podman, you will need to
restore the original file permissions in order to be able to manipulate the
`exercises/` directory normally again. This can be done with the following
command:

```bash
sudo chown -R $(id -u):$(id -g) exercises/
```


## Suggested workflow

The suggested workflow during the course is for you to have this course's
handouts opened in one browser tab, and the code editor opened in another tab.
If your screen is large enough, two browser windows side by side will provide
extra comfort.

After performing basic Visual Studio Code configuration, I advise opening a
terminal, which you can do by showing the bottom pane of the code editor using
the `Ctrl+J` keyboard shortcut.

You would then direct this terminal to the exercises directory if you're not
already there...

```bash
cd ~/exercises
```

And, when you're ready to do the exercises, start running `cargo run` with the
options specified by the corresponding course material.
