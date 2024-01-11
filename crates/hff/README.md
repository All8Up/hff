# HFF
Another file format.  The purpose of this format is to be a cross between IFF/RIFF formats and a zip type archive with some unique additions.  Where the differences lie are that unlike IFF/RIFF, the structure is moved up front for discoverability without scanning and that the structure is hierarchical in nature much like ZIP.  In fact, the primary testbed/tool ('hff') implements a simple pack/unpack subcommand which packages up the contents of a directory into a single hff file and can then unpack to a different location.  The command supports optional LZMA compression in the process.

The overall container format is intended to remain as non-opinionated as possible in line with IFF/RIFF formats that are used for many different file formats.  Other purposes of the format are for specific needs of the author which may or may not be generally useful but should not impact this container other than to make it more feature complete for others.

## Structure
The structure of an HFF is split into three pieces:

### Header
A fixed size structure describing the basic container.  This header is able to identify the data as being an HFF and the version of the container format contained within.  Additionally, the HFF header is used to detect which endian the overall structure was written with.  The endian mode of the file only applies to the structural descriptions and does not cause a change to the chunk data, determining how to encode/decode chunks is left to the user as HFF makes no specifications as to the content.

### Description
The second portion of the HFF container is built of two arrays.  The first array is a hierarchical structure of 'tables' which can be thought of as directories on disk.  Each table can have optional metadata attached, optional child tables (sub directories) and optional chunks (the files in the directory).  The second array is referenced by the tables and contains the information about each chunk owned by a given table.  The chunks (and metadata) are defined by the container as a set of bytes with a given length, it does not specify the content or in anyway interpret it.  The only specification provided is that the offset to the data and the data length are full 64 bit values allowing petabyte levels of storage if needed.

### Body
The actual metadata and chunk data stored in the file.  The only specification provided is that each chunk will start at a multiple of 16 bytes from the start of the file.

## Identification
The identification of  the HFF itself, tables and chunks is a bit overkill for many purposes.  The HFF itself can specify a 64 bit unique ID which can be used to determine how to interpret the content of the overall HFF.  This ID is found in the header for quick access without having to scan the content of the container.  The second type of ID which applies to both tables and chunks is a full 128 bit ID which can be, for the most part, any arbitrary value desired.  There are several default formats intended but not fully fleshed out at this time.  Currently, the intention is to support several combinations; UUID, dual eight character codes, fixed 16 byte string and whatever else is desired.  The UUID and dual ECC will be the primary implementations to start with.

NOTE: Arbitrary ID's are still being implemented.  At this time the API's expect two ECC's, like IFF/RIFF four character codes, just 8 characters each.

# HFF command line tool
The hff command line tool is currently a utility written as both a testbed and example of using the format.  It supplies 3 subcommands as follows (hff --help for more information):

## hff dump [input] <options>
The hff dump subcommand is used to inspect the structure of an HFF container.  It will, by default, simply print out the tables and information about them in depth first order.  There are options to also dump out the metadata and chunk information for each table.  In the case of metadata, there are also options to attempt to interpret the content as the two utility structures: Ksv and StringVec.

## hff pack [input] [output] <options>
Acts like a primitive variation of zip to scan the content of a directory and package it into a single archive HFF container.

## hff unpack [input] [output] <options>
Reverses the pack command and writes the original content of a HFF container back to disk.

# Status
The current status is still very much in a alpha though potentially more beta state at this time.  The std::io implementations are the primary focus and seem to be working.  The tokio/async-std implementations currently only support async read but they also 'seem' to work though there are no large tests at this time.  All tests are very development driven right now and rather poor, further and better tests are required.
