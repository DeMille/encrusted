NAME

CZECH - Comprehensive Z-machine Emulation CHecker

------------------------------------------------------------------------------

DESCRIPTION

Czech is a set of tests to determine whether a Z-machine interpreter is
compliant with the Z-machine Spec (currently 1.0, with some fixes in
1.1draft7).

Someday, Czech may contain a bunch of test files, but for now it's just one
Inform file you can compile (under different versions) and run.

Czech is designed to give information about which tests failed, to
make it easy to debug problems with your interpreter.

czech.inf itself is designed to run with no user IO. (We can still
test read/write opcodes using streams. Also, see TODO.)

czech.inf runs 425 tests under v5 as of version 0.8.

------------------------------------------------------------------------------

USAGE

- inf [-v3|-v4|-v5|-v8] czech.inf

- Run your interpreter of choice on czech.z[3458]

- Examine the output. 

- Compare to czech.out[3458] with your favorite file comparison tool, such as
  diff on Unix (and OS X I guess), or fc in the Windows command shell.
  Note that header information WILL be different for different interpreters,
  platforms, etc.

See czech.inf for usage notes (e.g., you can skip sets of tests) and
information on how to find out which test failed.

------------------------------------------------------------------------------

TODO

Lots!

- Test other opcodes

- Separate IO test file that asks the user to type things (to test
  keyboard/screen streams, time-dependent input, etc.)

- In theory, games should be able to create new objects, and change
  abbreviations (which will change strings in high memory, sort of!), global
  variables, etc., just using storeb. dynamic.inf, which is now just a stub,
  would test a whole bunch of sneaky ways to manipulate dynamic memory. 

- If we could work out licensing et al., I could include the fancier
  test scripts like terpetude in here to mak a true Z-machine test suite.

------------------------------------------------------------------------------

AUTHOR

Amir Karger (akarger@cpan.org)

------------------------------------------------------------------------------

LICENSE

Czech, v0.8
Copyright (C) 2003, Amir Karger

The copyright holder hereby grants the rights of usage, distribution
and modification of this software to everyone and for any purpose, as
long as this license and the copyright notice above are preserved and
not modified.  There is no warranty for this software.

------------------------------------------------------------------------------

HISTORY

Czech is based on Evin Robertson's test script for nitfol.

I developed it as a side project while working on Plotz, which translates a
Z-code file to Perl (and possibly other languages). I found out there wasn't
really a comprehensive test for Z-machine spec compliance, although there were
a number of programs that checked specific aspects of an interepreter (such as
terpetude).

I started with Evin's test script and removed a whole bunch of things that
my interpreter couldnt' do yet.  Over time, I added in other pieces of his
tests, and fleshed it out with many more tests, trying to find the nooks and
crannies of the Z-machine (indirect variables, signed numbers, ...)

See CHANGES.txt to see how the versions changed over time.
