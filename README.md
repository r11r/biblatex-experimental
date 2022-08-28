# \[WIP\] biblatex rust backend

Experimental tooling for a potential biblatex backend written in Rust.
At the moment this work is purely academic and out of personal curiosity.

Potential goals are to implement the [biber](https://github.com/plk/biber) 
toolchain in processing order.

Todos:
* [x] Raw BibTeX parsing (mostly)
* [ ] Support for Compound BibTeX values
* [ ] Macro replacement
* [ ] XDATA
* [ ] cross-references and sets
* [x] Datamodel creation from `.bcf`-file
* [ ] Datamodel validation
* [ ] ...
* [ ] ... lots more stuff (see [Biber Manual](https://ctan.mc1.root.project-creative.net/biblio/biber/base/documentation/biber.pdf) page 8) ...