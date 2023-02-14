Boilerplate for nrf9160

SPM must be installed on the nrf9160.

pip3 install --user -U west

Download nRF Command Line tools https://www.nordicsemi.com/Products/Development-tools/nRF-Command-Line-Tools/Download

apt install nrf-command-line-tools_**.deb

$ west init -m https://github.com/nrfconnect/sdk-nrf --mr v2.1.3 ncs
$ cd ncs
$ west update # This takes *ages*
$ cd nrf/examples/spm
$ west build --board=nrf9160dk_nrf9160
$ west flash


Setup for probe.rs
sudo apt install -y pkg-config libusb-1.0-0-dev libftdi1-dev libudev-dev

wget https://probe.rs/files/69-probe-rs.rules
mv 69-probe-rs.rules /etc/udev/rules.d
udevadm control --reload
udevadm trigger

cargo install cargo-embed
