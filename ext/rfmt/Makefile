RB_SYS_BUILD_DIR ?= /Users/fujitanisora/dev/oss/rfmt/ext/rfmt/.rb-sys
CARGO ?= cargo
CARGO_BUILD_TARGET ?= 
SOEXT ?= dylib


# Determine the prefix Cargo uses for the lib.
ifneq ($(SOEXT),dll)
	SOEXT_PREFIX ?= lib
endif

RB_SYS_CARGO_PROFILE ?= release
RB_SYS_CARGO_FEATURES ?= 
RB_SYS_GLOBAL_RUSTFLAGS ?= 
RB_SYS_EXTRA_RUSTFLAGS ?= 
RB_SYS_EXTRA_CARGO_ARGS ?= 
RB_SYS_CARGO_MANIFEST_DIR ?= .

# Set dirname for the profile, since the profiles do not directly map to target dir (i.e. dev -> debug)
ifeq ($(RB_SYS_CARGO_PROFILE),dev)
	RB_SYS_CARGO_PROFILE_DIR ?= debug
else
	RB_SYS_CARGO_PROFILE_DIR ?= $(RB_SYS_CARGO_PROFILE)
endif

# Set the build profile (dev, release, etc.).
	RB_SYS_CARGO_PROFILE_FLAG = --profile $(RB_SYS_CARGO_PROFILE)

# Account for sub-directories when using `--target` argument with Cargo
RB_SYS_CARGO_TARGET_DIR ?= target
ifneq ($(CARGO_BUILD_TARGET),)
	RB_SYS_FULL_TARGET_DIR = $(RB_SYS_CARGO_TARGET_DIR)/$(CARGO_BUILD_TARGET)
else
	RB_SYS_FULL_TARGET_DIR = $(RB_SYS_CARGO_TARGET_DIR)
endif

target_prefix = /rfmt
TARGET_NAME = rfmt
TARGET_ENTRY = Init_$(TARGET_NAME)
RUBYARCHDIR = $(sitearchdir)$(target_prefix)
TARGET = rfmt
DLLIB = $(TARGET).bundle
RUSTLIBDIR = $(RB_SYS_FULL_TARGET_DIR)/$(RB_SYS_CARGO_PROFILE_DIR)
RUSTLIB = $(RUSTLIBDIR)/$(SOEXT_PREFIX)$(TARGET_NAME).$(SOEXT)
TIMESTAMP_DIR = .
POSTLINK = dsymutil $@; { test -z '$(RUBY_CODESIGN)' || codesign -s '$(RUBY_CODESIGN)' -f $@; }

CLEANOBJS = $(RUSTLIBDIR) $(RB_SYS_BUILD_DIR)
CLEANLIBS = $(DLLIB) $(RUSTLIB)
RUBYGEMS_CLEAN_DIRS = $(CLEANOBJS) $(CLEANFILES) ./cargo-vendor


SHELL = /bin/sh

# V=0 quiet, V=1 verbose.  other values don't work.
V = 0
Q1 = $(V:1=)
Q = $(Q1:0=@)
ECHO1 = $(V:1=@ :)
ECHO = $(ECHO1:0=@ echo)
NULLCMD = :

#### Start of system configuration section. ####

srcdir = .
topdir = /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0
hdrdir = $(topdir)
arch_hdrdir = /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/universal-darwin23
PATH_SEPARATOR = :
VPATH = $(srcdir):$(arch_hdrdir)/ruby:$(hdrdir)/ruby

prefix = $(DESTDIR)/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr

rubysitearchprefix = $(rubylibprefix)/$(sitearch)

rubyarchprefix = $(rubylibprefix)/$(arch)

rubylibprefix = $(libdir)/$(RUBY_BASE_NAME)

exec_prefix = $(prefix)

vendorarchhdrdir = $(vendorhdrdir)/$(sitearch)

sitearchhdrdir = $(sitehdrdir)/$(sitearch)

rubyarchhdrdir = $(rubyhdrdir)/$(arch)

vendorhdrdir = $(rubyhdrdir)/vendor_ruby

sitehdrdir = $(rubyhdrdir)/site_ruby

rubyhdrdir = $(includedir)/$(RUBY_VERSION_NAME)

vendorarchdir = $(vendorlibdir)/$(sitearch)

vendorlibdir = $(vendordir)/$(ruby_version)

vendordir = $(rubylibprefix)/vendor_ruby

sitearchdir = $(sitelibdir)/$(sitearch)

sitelibdir = $(sitedir)/$(ruby_version)

sitedir = $(DESTDIR)/Library/Ruby/Site

rubyarchdir = $(rubylibdir)/$(arch)

rubylibdir = $(rubylibprefix)/$(ruby_version)

sitearchincludedir = $(includedir)/$(sitearch)

archincludedir = $(includedir)/$(arch)

sitearchlibdir = $(libdir)/$(sitearch)

archlibdir = $(libdir)/$(arch)

ridir = $(datarootdir)/$(RI_BASE_NAME)

mandir = $(DESTDIR)/usr/share/man

localedir = $(datarootdir)/locale

libdir = $(exec_prefix)/lib

psdir = $(docdir)

pdfdir = $(docdir)

dvidir = $(docdir)

htmldir = $(docdir)

infodir = $(DESTDIR)/usr/share/info

docdir = $(datarootdir)/doc/$(PACKAGE)

oldincludedir = $(DESTDIR)/usr/include

includedir = $(DESTDIR)/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk$(prefix)/include

runstatedir = $(localstatedir)/run

localstatedir = $(prefix)/var

sharedstatedir = $(prefix)/com

sysconfdir = $(DESTDIR)/Library/Ruby/Site

datadir = $(datarootdir)

datarootdir = $(prefix)/share

libexecdir = $(exec_prefix)/libexec

sbindir = $(exec_prefix)/sbin

bindir = $(exec_prefix)/bin

archdir = $(rubyarchdir)



CC_WRAPPER = 
CC = xcrun clang
CXX = xcrun clang++
LIBRUBY = $(LIBRUBY_SO)
LIBRUBY_A = lib$(RUBY_SO_NAME)-static.a
LIBRUBYARG_SHARED = -l$(RUBY_SO_NAME)
LIBRUBYARG_STATIC = -l$(RUBY_SO_NAME)-static -framework Security -framework Foundation $(MAINLIBS)
empty =
OUTFLAG = -o $(empty)
COUTFLAG = -o $(empty)
CSRCFLAG = $(empty)

RUBY_EXTCONF_H = 
cflags   = $(optflags) $(debugflags) $(warnflags)
cxxflags = $(optflags) $(debugflags) $(warnflags)
optflags = 
debugflags = -g
warnflags = 
cppflags = 
CCDLFLAGS = 
CFLAGS   = $(CCDLFLAGS)  -g -Os -pipe -DHAVE_GCC_ATOMIC_BUILTINS -DUSE_FFI_CLOSURE_ALLOC $(ARCH_FLAG)
INCFLAGS = -I. -I$(arch_hdrdir) -I$(hdrdir)/ruby/backward -I$(hdrdir) -I$(srcdir)
DEFS     = 
CPPFLAGS =  -D_XOPEN_SOURCE -D_DARWIN_C_SOURCE -D_DARWIN_UNLIMITED_SELECT -D_REENTRANT $(DEFS) $(cppflags)
CXXFLAGS = $(CCDLFLAGS)  -g -Os -pipe  $(ARCH_FLAG)
ldflags  = -L. -L/AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.7.Internal.sdk/usr/local/lib
dldflags = $(ARCH_FLAG)             -undefined dynamic_lookup 
ARCH_FLAG = 
DLDFLAGS = $(ldflags) $(dldflags) $(ARCH_FLAG)
LDSHARED = $(CC) -dynamic -bundle
LDSHAREDXX = $(CXX) -dynamic -bundle
AR = libtool -static
EXEEXT = 


RUBY_INSTALL_NAME = $(RUBY_BASE_NAME)

RUBY_SO_NAME = ruby.2.6

RUBYW_INSTALL_NAME = 

RUBY_VERSION_NAME = $(RUBY_BASE_NAME)-$(ruby_version)

RUBYW_BASE_NAME = rubyw

RUBY_BASE_NAME = ruby


arch = universal-darwin23
sitearch = $(arch)
ruby_version = 2.6.0
ruby = $(bindir)/$(RUBY_BASE_NAME)
RUBY = $(ruby)
ruby_headers = $(hdrdir)/ruby.h $(hdrdir)/ruby/backward.h $(hdrdir)/ruby/ruby.h $(hdrdir)/ruby/defines.h $(hdrdir)/ruby/missing.h $(hdrdir)/ruby/intern.h $(hdrdir)/ruby/st.h $(hdrdir)/ruby/subst.h $(arch_hdrdir)/ruby/config.h

RM = rm -f
RM_RF = $(RUBY) -run -e rm -- -rf
RMDIRS = rmdir -p
MAKEDIRS = mkdir -p
INSTALL = /usr/bin/install -c
INSTALL_PROG = $(INSTALL) -m 0755
INSTALL_DATA = $(INSTALL) -m 644
COPY = cp
TOUCH = exit >

#### End of system configuration section. ####

preload = 

CLEANFILES = mkmf.log
DISTCLEANFILES = 

all static install-rb: Makefile
	@$(NULLCMD)
.PHONY: all static install-rb
.PHONY: clean clean-so clean-static clean-rb



clean-static::
clean-rb-default::
clean-rb::
clean-so::
clean: clean-so clean-static clean-rb-default clean-rb
		-$(Q)$(RM) $(CLEANLIBS) $(CLEANOBJS) $(CLEANFILES) .*.time

distclean-rb-default::
distclean-rb::
distclean-so::
distclean-static::
distclean: clean distclean-so distclean-static distclean-rb-default distclean-rb
		-$(Q)$(RM) Makefile $(RUBY_EXTCONF_H) conftest.* mkmf.log
		-$(Q)$(RM) core ruby$(EXEEXT) *~ $(DISTCLEANFILES)
		-$(Q)$(RMDIRS) $(DISTCLEANDIRS) 2> /dev/null || true

realclean: distclean


.PHONY: gemclean

ifneq ($(RB_SYS_VERBOSE),)
	Q = $(0=@)
endif

export RBCONFIG_DESTDIR := 
export RBCONFIG_MAJOR := 2
export RBCONFIG_MINOR := 6
export RBCONFIG_TEENY := 10
export RBCONFIG_PATCHLEVEL := 210
export RBCONFIG_INSTALL := /usr/bin/install -c
export RBCONFIG_EXEEXT := 
export RBCONFIG_prefix := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr
export RBCONFIG_ruby_install_name := ruby
export RBCONFIG_RUBY_INSTALL_NAME := ruby
export RBCONFIG_RUBY_SO_NAME := ruby.2.6
export RBCONFIG_exec := exec
export RBCONFIG_ruby_pc := ruby-2.6.pc
export RBCONFIG_CC_WRAPPER := 
export RBCONFIG_PACKAGE := ruby
export RBCONFIG_BUILTIN_TRANSSRCS := enc/trans/newline.c
export RBCONFIG_MANTYPE := doc
export RBCONFIG_vendorarchhdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/vendor_ruby/universal-darwin23
export RBCONFIG_sitearchhdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/site_ruby/universal-darwin23
export RBCONFIG_rubyarchhdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/universal-darwin23
export RBCONFIG_vendorhdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/vendor_ruby
export RBCONFIG_sitehdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0/site_ruby
export RBCONFIG_rubyhdrdir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/ruby-2.6.0
export RBCONFIG_RUBY_SEARCH_PATH := 
export RBCONFIG_UNIVERSAL_INTS := 'long long' long int short
export RBCONFIG_UNIVERSAL_ARCHNAMES := arm64e=arm64e x86_64=x86_64
export RBCONFIG_configure_args := '--prefix=/usr' '--mandir=/usr/share/man' '--infodir=/usr/share/info' '--disable-dependency-tracking' '--prefix=/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr' '--sysconfdir=/Library/Ruby/Site' '--with-sitedir=/Library/Ruby/Site' '--enable-shared' '--with-arch=arm64e,x86_64' '--with-openssl-dir=/AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.7.Internal.sdk/usr/local/libressl-by-deployment-target/13.0' '--without-ext=tk' '--disable-silent-rules' 'ac_cv_func_getcontext=no' 'ac_cv_func_setcontext=no' 'ac_cv_func_utimensat=no' 'ac_cv_c_compiler_gnu=no' 'ac_cv_header_net_if_h=yes' 'av_cv_header_ifaddrs_h=yes' 'rb_cv_pri_prefix_long_long=ll' 'rb_cv_stack_grow_dir_arm64e=-1' 'ac_cv_sizeof_struct_stat_st_size=SIZEOF_OFF_T' 'ac_cv_sizeof_struct_stat_st_blocks=SIZEOF_INT64_T' 'ac_cv_sizeof_struct_stat_st_ino=SIZEOF_UINT64_T' 'CC=/AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Toolchains/OSX14.7.xctoolchain/usr/bin/cc' 'CFLAGS=-arch arm64e -arch x86_64 -g -Os -pipe -DHAVE_GCC_ATOMIC_BUILTINS -DUSE_FFI_CLOSURE_ALLOC' 'LDFLAGS=-arch arm64e -arch x86_64            ' 'CXX=/AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Toolchains/OSX14.7.xctoolchain/usr/bin/c++' 'CXXFLAGS=-arch arm64e -arch x86_64 -g -Os -pipe '
export RBCONFIG_CONFIGURE := configure
export RBCONFIG_vendorarchdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/vendor_ruby/2.6.0/universal-darwin23
export RBCONFIG_vendorlibdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/vendor_ruby/2.6.0
export RBCONFIG_vendordir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/vendor_ruby
export RBCONFIG_sitearchdir := /Library/Ruby/Site/2.6.0/universal-darwin23
export RBCONFIG_sitelibdir := /Library/Ruby/Site/2.6.0
export RBCONFIG_sitedir := /Library/Ruby/Site
export RBCONFIG_rubyarchdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/2.6.0/universal-darwin23
export RBCONFIG_rubylibdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/2.6.0
export RBCONFIG_ruby_version := 2.6.0
export RBCONFIG_sitearch := universal-darwin23
export RBCONFIG_arch := universal-darwin23
export RBCONFIG_sitearchincludedir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/universal-darwin23
export RBCONFIG_archincludedir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include/universal-darwin23
export RBCONFIG_sitearchlibdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/universal-darwin23
export RBCONFIG_archlibdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/universal-darwin23
export RBCONFIG_libdirname := libdir
export RBCONFIG_RUBY_EXEC_PREFIX := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr
export RBCONFIG_RUBY_LIB_VERSION := 
export RBCONFIG_RUBY_LIB_VERSION_STYLE := 3	/* full */
export RBCONFIG_RI_BASE_NAME := ri
export RBCONFIG_ridir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/ri
export RBCONFIG_rubysitearchprefix := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/universal-darwin23
export RBCONFIG_rubyarchprefix := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/universal-darwin23
export RBCONFIG_MAKEFILES := Makefile GNUmakefile
export RBCONFIG_PLATFORM_DIR := 
export RBCONFIG_THREAD_MODEL := pthread
export RBCONFIG_SYMBOL_PREFIX := _
export RBCONFIG_EXPORT_PREFIX := 
export RBCONFIG_COMMON_HEADERS := 
export RBCONFIG_COMMON_MACROS := 
export RBCONFIG_COMMON_LIBS := 
export RBCONFIG_MAINLIBS := -lpthread -ldl -lobjc
export RBCONFIG_ENABLE_SHARED := yes
export RBCONFIG_DLDSHARED := xcrun clang -dynamiclib
export RBCONFIG_DLDLIBS := 
export RBCONFIG_SOLIBS := -lpthread -ldl -lobjc
export RBCONFIG_LIBRUBYARG_SHARED := -lruby.2.6
export RBCONFIG_LIBRUBYARG_STATIC := -lruby.2.6-static -framework Security -framework Foundation -lpthread -ldl -lobjc
export RBCONFIG_LIBRUBYARG := -lruby.2.6
export RBCONFIG_LIBRUBY := libruby.2.6.dylib
export RBCONFIG_LIBRUBY_ALIASES := libruby.dylib
export RBCONFIG_LIBRUBY_SONAME := libruby.2.6.dylib
export RBCONFIG_LIBRUBY_SO := libruby.2.6.dylib
export RBCONFIG_LIBRUBY_A := libruby.2.6-static.a
export RBCONFIG_RUBYW_INSTALL_NAME := 
export RBCONFIG_rubyw_install_name := 
export RBCONFIG_EXTDLDFLAGS := 
export RBCONFIG_EXTLDFLAGS := 
export RBCONFIG_strict_warnflags := 
export RBCONFIG_warnflags := 
export RBCONFIG_debugflags := -g
export RBCONFIG_optflags := 
export RBCONFIG_NULLCMD := :
export RBCONFIG_ENABLE_DEBUG_ENV := 
export RBCONFIG_DLNOBJ := dln.o
export RBCONFIG_INSTALL_STATIC_LIBRARY := no
export RBCONFIG_MJIT_SUPPORT := yes
export RBCONFIG_EXECUTABLE_EXTS := 
export RBCONFIG_ARCHFILE := 
export RBCONFIG_LIBRUBY_RELATIVE := no
export RBCONFIG_EXTOUT := .ext
export RBCONFIG_PREP := miniruby
export RBCONFIG_CROSS_COMPILING := no
export RBCONFIG_TEST_RUNNABLE := yes
export RBCONFIG_rubylibprefix := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby
export RBCONFIG_setup := Setup
export RBCONFIG_ENCSTATIC := 
export RBCONFIG_EXTSTATIC := 
export RBCONFIG_STRIP := strip -A -n
export RBCONFIG_SOEXT := dylib
export RBCONFIG_TRY_LINK := 
export RBCONFIG_PRELOADENV := DYLD_INSERT_LIBRARIES
export RBCONFIG_LIBPATHENV := DYLD_FALLBACK_LIBRARY_PATH
export RBCONFIG_RPATHFLAG := 
export RBCONFIG_LIBPATHFLAG := -L%s
export RBCONFIG_LINK_SO := \ndsymutil $@; { test -z '$(RUBY_CODESIGN)' || codesign -s '$(RUBY_CODESIGN)' -f $@; }
export RBCONFIG_ASMEXT := S
export RBCONFIG_LIBEXT := a
export RBCONFIG_DLEXT2 := 
export RBCONFIG_DLEXT := bundle
export RBCONFIG_LDSHAREDXX := xcrun clang++ -dynamic -bundle
export RBCONFIG_LDSHARED := xcrun clang -dynamic -bundle
export RBCONFIG_CCDLFLAGS := 
export RBCONFIG_STATIC := 
export RBCONFIG_ARCH_FLAG := 
export RBCONFIG_DLDFLAGS := -undefined dynamic_lookup
export RBCONFIG_ALLOCA := 
export RBCONFIG_MATHN := yes
export RBCONFIG_dsymutil := dsymutil
export RBCONFIG_codesign := codesign
export RBCONFIG_POSTLINK := dsymutil $@; { test -z '$(RUBY_CODESIGN)' || codesign -s '$(RUBY_CODESIGN)' -f $@; }
export RBCONFIG_WERRORFLAG := 
export RBCONFIG_CHDIR := cd -P
export RBCONFIG_RMALL := rm -fr
export RBCONFIG_RMDIRS := rmdir -p
export RBCONFIG_RMDIR := rmdir
export RBCONFIG_CP := cp
export RBCONFIG_RM := rm -f
export RBCONFIG_PKG_CONFIG := 
export RBCONFIG_DOXYGEN := 
export RBCONFIG_DOT := 
export RBCONFIG_MAKEDIRS := mkdir -p
export RBCONFIG_MKDIR_P := mkdir -p
export RBCONFIG_INSTALL_DATA := /usr/bin/install -c -m 644
export RBCONFIG_INSTALL_SCRIPT := /usr/bin/install -c
export RBCONFIG_INSTALL_PROGRAM := /usr/bin/install -c
export RBCONFIG_SET_MAKE := 
export RBCONFIG_LN_S := ln -s
export RBCONFIG_NM := nm
export RBCONFIG_DLLWRAP := 
export RBCONFIG_WINDRES := 
export RBCONFIG_OBJCOPY := 
export RBCONFIG_OBJDUMP := objdump
export RBCONFIG_ASFLAGS := 
export RBCONFIG_AS := as
export RBCONFIG_ARFLAGS := -no_warning_for_no_symbols -o
export RBCONFIG_AR := libtool -static
export RBCONFIG_RANLIB := :
export RBCONFIG_try_header := try_compile
export RBCONFIG_CC_VERSION_MESSAGE := Apple clang version 15.0.0 (clang-1500.3.9.1) [+internal-os]\nTarget: x86_64-apple-darwin22.1.0\nThread model: posix\nInstalledDir: /AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Toolchains/OSX14.7.xctoolchain/usr/bin
export RBCONFIG_CC_VERSION := xcrun clang --version
export RBCONFIG_MJIT_CC := /AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Toolchains/OSX14.7.xctoolchain/usr/bin/cc
export RBCONFIG_CSRCFLAG := 
export RBCONFIG_COUTFLAG := -o
export RBCONFIG_OUTFLAG := -o
export RBCONFIG_CPPOUTFILE := -o conftest.i
export RBCONFIG_GNU_LD := no
export RBCONFIG_LD := ld
export RBCONFIG_GCC := 
export RBCONFIG_EGREP := /usr/bin/grep -E
export RBCONFIG_GREP := /usr/bin/grep
export RBCONFIG_CPP := xcrun clang -E
export RBCONFIG_CXXFLAGS := -g -Os -pipe
export RBCONFIG_CXX := xcrun clang++
export RBCONFIG_OBJEXT := o
export RBCONFIG_CPPFLAGS := -D_XOPEN_SOURCE -D_DARWIN_C_SOURCE -D_DARWIN_UNLIMITED_SELECT -D_REENTRANT
export RBCONFIG_LDFLAGS := -L. -L/AppleInternal/Library/BuildRoots/289ffcb4-455d-11ef-953d-e2437461156c/Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.7.Internal.sdk/usr/local/lib
export RBCONFIG_CFLAGS := -g -Os -pipe -DHAVE_GCC_ATOMIC_BUILTINS -DUSE_FFI_CLOSURE_ALLOC
export RBCONFIG_CC := xcrun clang
export RBCONFIG_target_os := darwin23
export RBCONFIG_target_vendor := apple
export RBCONFIG_target_cpu := universal
export RBCONFIG_target := universal-apple-darwin23
export RBCONFIG_host_os := darwin23
export RBCONFIG_host_vendor := apple
export RBCONFIG_host_cpu := aarch64
export RBCONFIG_host := aarch64-apple-darwin23
export RBCONFIG_RUBY_VERSION_NAME := ruby-2.6.0
export RBCONFIG_RUBYW_BASE_NAME := rubyw
export RBCONFIG_RUBY_BASE_NAME := ruby
export RBCONFIG_build_os := darwin23
export RBCONFIG_build_vendor := apple
export RBCONFIG_build_cpu := x86_64
export RBCONFIG_build := x86_64-apple-darwin23
export RBCONFIG_RUBY_API_VERSION := 2.6
export RBCONFIG_RUBY_PROGRAM_VERSION := 2.6.10
export RBCONFIG_HAVE_GIT := yes
export RBCONFIG_GIT := git
export RBCONFIG_cxxflags := -g
export RBCONFIG_cppflags := 
export RBCONFIG_cflags := -g
export RBCONFIG_target_alias := 
export RBCONFIG_host_alias := 
export RBCONFIG_build_alias := 
export RBCONFIG_LIBS := 
export RBCONFIG_ECHO_T := 
export RBCONFIG_ECHO_N := 
export RBCONFIG_ECHO_C := \\c
export RBCONFIG_DEFS := 
export RBCONFIG_mandir := /usr/share/man
export RBCONFIG_localedir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/locale
export RBCONFIG_libdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib
export RBCONFIG_psdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/doc/ruby
export RBCONFIG_pdfdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/doc/ruby
export RBCONFIG_dvidir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/doc/ruby
export RBCONFIG_htmldir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/doc/ruby
export RBCONFIG_infodir := /usr/share/info
export RBCONFIG_docdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share/doc/ruby
export RBCONFIG_oldincludedir := /usr/include
export RBCONFIG_includedir := /Applications/Xcode.app/Contents/Developer/Platforms/MacOSX.platform/Developer/SDKs/MacOSX14.5.sdk/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/include
export RBCONFIG_runstatedir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/var/run
export RBCONFIG_localstatedir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/var
export RBCONFIG_sharedstatedir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/com
export RBCONFIG_sysconfdir := /Library/Ruby/Site
export RBCONFIG_datadir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share
export RBCONFIG_datarootdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/share
export RBCONFIG_libexecdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/libexec
export RBCONFIG_sbindir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/sbin
export RBCONFIG_bindir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/bin
export RBCONFIG_exec_prefix := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr
export RBCONFIG_PACKAGE_URL := 
export RBCONFIG_PACKAGE_BUGREPORT := 
export RBCONFIG_PACKAGE_STRING := 
export RBCONFIG_PACKAGE_VERSION := 
export RBCONFIG_PACKAGE_TARNAME := 
export RBCONFIG_PACKAGE_NAME := 
export RBCONFIG_PATH_SEPARATOR := :
export RBCONFIG_SHELL := /bin/sh
export RBCONFIG_UNICODE_VERSION := 12.1.0
export RBCONFIG_UNICODE_EMOJI_VERSION := 12.0
export RBCONFIG_SDKROOT := 
export RBCONFIG_archdir := /System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib/ruby/2.6.0/universal-darwin23
export RBCONFIG_topdir := .
export RBCONFIG_srcdir := .
export RUSTFLAGS := $(RB_SYS_GLOBAL_RUSTFLAGS) $(RB_SYS_EXTRA_RUSTFLAGS) $(RUSTFLAGS)

FORCE: ;

RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN ?= false

# Only run if the we are told to explicitly install the Rust toolchain
ifneq ($(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN),false)
RB_SYS_RUSTUP_PROFILE ?= minimal

# If the user passed true, we assume stable Rust. Otherwise, use what
# was specified (i.e. RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN=beta)
ifeq ($(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN),true)
  RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN = stable
endif

# If a $RUST_TARGET is specified (i.e. for rake-compiler-dock), append
# that to the profile.
ifeq ($(RUST_TARGET),)
  RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)
else
  RB_SYS_DEFAULT_TOOLCHAIN = $(RB_SYS_FORCE_INSTALL_RUST_TOOLCHAIN)-$(RUST_TARGET)
endif

# Since we are forcing the installation of the Rust toolchain, we need
# to set these env vars unconditionally for the build.
export CARGO_HOME := $(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/cargo
export RUSTUP_HOME := $(RB_SYS_BUILD_DIR)/$(RB_SYS_DEFAULT_TOOLCHAIN)/rustup
export PATH := $(CARGO_HOME)/bin:$(RUSTUP_HOME)/bin:$(PATH)
export RUSTUP_TOOLCHAIN := $(RB_SYS_DEFAULT_TOOLCHAIN)
export CARGO := $(CARGO_HOME)/bin/cargo


$(CARGO):
	$(Q) $(MAKEDIRS) $(CARGO_HOME) $(RUSTUP_HOME)
	$(Q) curl --proto '=https' --tlsv1.2 --retry 10 --retry-connrefused -fsSL "https://sh.rustup.rs" | sh -s -- --no-modify-path --profile $(RB_SYS_RUSTUP_PROFILE) --default-toolchain none -y
	$(Q) $(CARGO_HOME)/bin/rustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE) || (sleep 5; $(Q) $(CARGO_HOME)/bin/rustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE)) || (sleep 5; $(Q) $(CARGO_HOME)/bin/rustup toolchain install $(RB_SYS_DEFAULT_TOOLCHAIN) --profile $(RB_SYS_RUSTUP_PROFILE))
	$(Q) $(CARGO_HOME)/bin/rustup default $(RB_SYS_DEFAULT_TOOLCHAIN) || (sleep 5; $(Q) $(CARGO_HOME)/bin/rustup default $(RB_SYS_DEFAULT_TOOLCHAIN)) || (sleep 5; $(Q) $(CARGO_HOME)/bin/rustup default $(RB_SYS_DEFAULT_TOOLCHAIN))


$(RUSTLIB): $(CARGO)
endif


$(TIMESTAMP_DIR)/.sitearchdir.time:
	$(Q) $(MAKEDIRS) $(@D) $(RUBYARCHDIR)
	$(Q) $(TOUCH) $@

$(RUSTLIB): FORCE
	$(ECHO) generating $(@) \("$(RB_SYS_CARGO_PROFILE)"\)
	$(CARGO) rustc $(RB_SYS_EXTRA_CARGO_ARGS) --manifest-path $(RB_SYS_CARGO_MANIFEST_DIR)/Cargo.toml --target-dir $(RB_SYS_CARGO_TARGET_DIR) --lib $(RB_SYS_CARGO_PROFILE_FLAG) -- -C linker=xcrun -C link-arg=clang -L native=/System/Library/Frameworks/Ruby.framework/Versions/2.6/usr/lib -C link-arg=-undefined -C link-arg=dynamic_lookup -C link-arg=-Wl,-undefined,dynamic_lookup

$(DLLIB): $(RUSTLIB)
	$(Q) $(COPY) "$(RUSTLIB)" $@
	$(Q) $(POSTLINK)

install-so: $(DLLIB) $(TIMESTAMP_DIR)/.sitearchdir.time
	$(ECHO) installing $(DLLIB) to $(RUBYARCHDIR)
	$(Q) install_name_tool -id "" $(DLLIB)
	$(Q) $(MAKEDIRS) $(RUBYARCHDIR)
	$(INSTALL_PROG) $(DLLIB) $(RUBYARCHDIR)

gemclean: install-so
	$(ECHO) Cleaning gem artifacts
	-$(Q)$(RM_RF) $(RUBYGEMS_CLEAN_DIRS) 2> /dev/null || true

install: install-so

all: $(DLLIB)
