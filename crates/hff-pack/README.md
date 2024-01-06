# Hffpack
A command line tool to package multiple hff files into a single package.  While the goal is specific to HFF and loose HFF packaging, it does not prevent usage for more generic options like a zip or tarball.

## Ecc's Specific to HFF Archives
There are 3 primary Ecc's in use:

`_ARCHIVE`: The outer container table type of the archive.  All of the packaged items will exist under an outer table of this type.
`_DIRENT `: Children tables which represent folders in the hierarchy of the archive.
`_FILEENT`: A raw file entry for something which was not an HFF source file.  Basically, anything which does not start with an HFF_MAGIC Ecc and/or is not a valid HFF.

## Metadata
Metadata is used to record the original file names and structure of what was packaged.  Within a `_DIRENT` table, this will be a simple table of strings where the first line is the name of the directory the table represents and the remaining lines are the file names of what the chunks consist of.