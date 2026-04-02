# XastGE Scene CLI Tool

## Install on deb-like

1. Download **embree 3.8.0** from here: https://github.com/embree/embree/releases/download/v3.8.0/embree-3.8.0.x86_64.rpm.tar.gz
2. Write these commands:
```sh
tar xzf embree-3.8.0.x86_64.rpm.tar.gz

sudo apt-get install alien dpkg-dev debhelper build-essential

sudo alien embree3-lib-3.8.0-1.x86_64.rpm
sudo alien embree3-devel-3.8.0-1.noarch.rpm
sudo alien embree3-examples-3.8.0-1.x86_64.rpm

sudo dpkg -i embree3-lib_3.8.0-2_amd64.deb
sudo dpkg -i embree3-devel_3.8.0-2_all.deb
sudo dpkg -i embree3-examples_3.8.0-2_amd64.deb

sudo apt-get install libtbb-dev
```
3. Download **OIDN** from here: https://github.com/RenderKit/oidn/releases/download/v2.4.1/oidn-2.4.1.x86_64.linux.tar.gz
4. Install to `/opt/odin`
5. Add to `.bashrc`:
```sh
export LD_LIBRARY_PATH=/opt/oidn/lib:$LD_LIBRARY_PATH
export PATH=/opt/oidn/bin:$PATH
export OIDN_DIR=/opt/oidn
```
6. Run `cargo run` (finally!)