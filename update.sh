#!/bin/bash

set -e

DEV_FILE=/dev/sdc1
MOUNT_DIR=/mnt

make
sudo mount $DEV_FILE $MOUNT_DIR
sudo cp kernel8.img $MOUNT_DIR
sudo sync $MOUNT_DIR
sudo umount $MOUNT_DIR
exit