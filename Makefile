KDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

export RUSTFLAGS = -Dwarnings

all:
	$(MAKE) -C $(KDIR) M=$(PWD) modules

clean:
	$(MAKE) -C $(KDIR) M=$(PWD) clean

install: all
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install
	depmod -a
