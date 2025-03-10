obj-m := driver_verifier.o

driver_verifier-objs := driver_verifier_core.o target/release/libdriver_verifier.a

EXTRA_LDFLAGS += --whole-archive $(src)/target/release/libdriver_verifier.a --no-whole-archive

KDIR ?= /lib/modules/$(shell uname -r)/build
