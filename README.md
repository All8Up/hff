# hff

A hierarchical file format intended to be a non-opinionated container.  Hff is intended to be a relatively simple data format which contains arbitrary data.  While there are existing formats with similar goals they were generally: not written in Rust, had too much specificity of purpose or were overly complicated for my needs.  It is quite possible I missed something which would have fit but since I've written these multiple times, it was a nice little side project.

**Goals**:

* Stay relatively simple.  Writing is a bit more complex than say IFF/RIFF containers but otherwise usage is pretty simple.
* Hiearchical as the name suggests.  It's basically tables of child tables and chunk data as deep as you might want to go.
* Move all structural data to the head of the file.  A quick set of reads from the front of the file and you have all the information needed to randomly access the content.  Or, you can read the entire thing, up to the user.
* Endian aware structure.  When writing the user can specify a specific endian or the machine native endian.  When reading, the 'structure' is automatically read in the proper form.
* Undefined chunk content.  There are no limits as to what can be put in them, what format etc.
* Minimize any form of specifics.  Internally the structure of the file is a header, table definition and chunk definition, otherwise the only fixed item is an 'identifier' for invalid (i.e. zero).  Anything else is completely users defined.

**Non-goals**:

* In general, there is only one rule, there should be no rules or specificity in the format.  Making it better for more use cases is viable up and until it has to become less general for other use cases.
