KDIR ?= /lib/modules/$(shell uname -r)/build
PWD := $(shell pwd)

all: rust_lib
	$(MAKE) -C $(KDIR) M=$(PWD) modules

rust_lib:
	cargo build --release

clean:
	cargo clean
	$(MAKE) -C $(KDIR) M=$(PWD) clean

install: all
	$(MAKE) -C $(KDIR) M=$(PWD) modules_install
	depmod -a
