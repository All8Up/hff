# hff-std
Synchronous implementation of reader, visitor, writer utilities for HFF.

Also supplies various shared elements such as the table/chunk builders for async variations.

# TODO
* Current implementation needs cleanup.
* Current read from chunks allocates a vector, switch to a model which expects the user to provide a properly sized buffer.
* Investigate a no_std version which supplies a minimal reader but almost certainly not a writer.
* Better testing.
* Look for a better way to supply chunk data source 'write' functionality.  Right now it uses a generic trait around the source in the builder and then uses a TryInto that has to know all of the underlying source types which is ... annoying.
