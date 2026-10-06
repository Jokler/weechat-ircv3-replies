installdir=$(HOME)/.weechat
testdir=./test_dir

.PHONY: all install install_test test run format clippy
all: src/*
	cargo build --release

all_debug: src/*
	cargo build

install: all | $(installdir)/plugins
	cp target/release/libweechat_ircv3_replies.so $(installdir)/plugins

install_test: all_debug | $(testdir)/plugins
	cp target/debug/libweechat_ircv3_replies.so $(testdir)/plugins

run: install
	weechat -a

test: install_test
	weechat -a -d $(testdir)

$(installdir):
	mkdir $@

$(installdir)/plugins: | $(installdir)
	mkdir $@

$(testdir):
	mkdir $@

$(testdir)/plugins: | $(testdir)
	mkdir $@

format:
	cargo fmt -- --write-mode=overwrite
	clang-format -style=mozilla -i src/*.c
