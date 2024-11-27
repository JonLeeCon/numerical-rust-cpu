# Linux

## System packages

There are many different Linux distributions, that each use different package managemers and name packages in a different way. The following commands should cover the most common Linux distributions, feel free to suggest your favorite distribution if it is not covered.

### Debian/Ubuntu family

```bash
sudo apt-get update  \
&& sudo apt-get install build-essential ca-certificates curl  \
                        libhdf5-dev libhwloc-dev libudev-dev pkg-config  \
                        unzip util-linux wget
```

### Fedora family

```bash
sudo dnf makecache --refresh  \
&& sudo dnf group install c-development  \
&& sudo dnf install ca-certificates curl \
                    hdf5-devel hwloc-devel libudev-devel pkg-config \
                    unzip util-linux wget
```

### RHEL family

```bash
sudo dnf makecache --refresh  \
&& sudo dnf groupinstall "Devlopment tools"  \
&& sudo dnf install epel-release  \
&& sudo /usr/bin/crb enable  \
&& sudo dnf makecache --refresh  \
&& sudo dnf install ca-certificates curl \
                    hdf5-devel hwloc-devel libudev-devel pkg-config \
                    unzip util-linux wget
```

### Arch family

```bash
sudo pacman -Sy  \
&& sudo pacman -S base-devel ca-certificates curl  \
                  libhdf5 libhwloc pkg-config  \
                  unzip util-linux wget
```

### openSUSE family

```bash
sudo zypper ref  \
&& sudo zypper in -t pattern devel_C_C++  \
&& sudo zypper in ca-certificates curl \
                  hdf5-devel hwloc-devel libudev-devel pkg-config \
                  unzip util-linux wget
```


{{#include unix.md}}
