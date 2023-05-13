# SPDX-License-Identifier: MIT
# Copyright (C) 2023 Akira Moroo

KDIR = /lib/modules/`uname -r`/build

kbuild:
	make -C $(KDIR) M=`pwd`

clean:
	make -C $(KDIR) M=`pwd` clean
