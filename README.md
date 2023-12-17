# hff

A hierarchical file format intended to be a non-opinionated container.

**Goals**:

* Keep it simple and not much more complicated than something like IFF/RIFF formats.
* Don't care what is put into the 'chunks' within the file.  With only two exceptions, there are no predefined identifiers or requirements.
* Move discoverability to the front of the format so scanning the content to find things is not needed.  Specifically, the primary goal is a container for randomly accessing the content instead of being read in all at once.  (The specific intention was an archive format somewhat like ZIP/7z etc but with different goals.)
* All lengths and sizes are u64 so there are no 4GB limitations.
* Support little and big endian within the structure.
* Update alignment within the file for current generation CPU's.  Specifically this means that everything in the format will be 16 byte aligned such that direct mapping within something like an mmap reader will not have to use unaligned (slower) instructions for contained SIMD data.
* Increase the identifier space.  Specifically IFF/RIFF use 4 character codes, I doubled that for more readability.  Later, when the 16 byte alignment of the internals was finalized, I had another 8 bytes free, so each chunk and table ended up with a primary and secondary ID.  The purpose and utility of this is up to the user but there are a few benefits which will be covered later.
* Metadata supported as a specific feature.  Metadata is in nearly every container at this point and rather than leaving it as an after-thought and up to the user, it is built into each 'table' within the format.

## Structure
The file structure broken into four components as follows:

### Header
The header contains basic information about the content of the HFF file.  The items in the header are:
* Identifier indicating this is a valid HFF file.  Also, the identifier can be in little or big endian indicating the endianness of the file structure portions.
* A version identifier in case of structural changes in the future.
* A single ECC (eight character code) to describe the overal content of the file.  Intended to differentiate between say text and image data.  (The file extensions should probably be used to do this at a high level also but that is of course not enforced.)
* Structural information, specifically how many tables are there and how many chunks are there.

### Tables
This is an array of table entries, the count found in the header indicates the length.  Each entry is 48 bytes (3 x 16) to maintain alignment and contains:
* Primary and secondary identification.  In 'most' cases, for a simple data file, the secondary is not used and should be set to Ecc::INVALID (i.e. all zero's).  (For the authors purposes, this is likely enforced but it is not a part of the generic container.)
* Optional metadata length and offset.  All offsets are absolute from the start of the file.
* A count of child tables under this table and an offset (index'd) where the next sibling table exists (or zero if no siblings).
* A count and index to the chunk data attached to the table.  (2 billion chunk limit in the overall file.)

### Chunks
A second array immediately following the table array, the count found in the header indicates the length.  Each entry is 32 bytes (2 x 16) to maintain alignment and contains:
* Primary and secondary identification.  The secondary ID is generally used to indicate special attributes of the contained chunk data such as a compression algorithm which was used to store the data.  Once again, this is not enforced in anyway, it is just the purpose for the author's use case.
* The length and offset within the file of the chunk data.  All chunk data is padded to 16 bytes, the length only specifies the real length before padding.  All offsets are absolute from the start of the file.

### Data
Both the metadata and chunk data is stored after the above chunk array.  The only limitation is 64 bit size and offset and also the offset must be a multiple of 16.

## Identifiers
The identification of tables and chunks is an eight character code.  This is a little missleading in terms that FCC (four character codes in IFF/RIFF) were a specific byte sequence, HFF uses the u64 value represented by the string when decoded into a [u8; 8] array.  There was no specific reason for this change but it does mean the ID's do not have to be valid utf8, they can be simple numbers if desired.  The only reserved ID is the zero value, everything else is up to use case.  NOTE: The implication of serializing the u64 is that it is endian swapped on little endian machines, so there is a little additional overhead if the structure is in the opposing endian to the local CPU.

# Examples
TODO
