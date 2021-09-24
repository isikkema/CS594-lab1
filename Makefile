EXE=target/release/j-grangle
TEAPOT=obj/teapot.obj
BUNNY=obj/bunny.obj
RUN=cargo run --quiet --release --
CONVERT=| jgraph -P | convert -

all: teapot.jpg teapot-hd.jpg teapot-lines.jpg teapot-points.jpg teapot-background.jpg teapot-solid.jpg bunny.jpg bunny-scale.jpg bunny-stretch.jpg bunny-translate.jpg bunny-rotate.jpg

teapot.jpg: $(EXE)
	$(RUN) $(TEAPOT) 100 100 $(CONVERT) $@

teapot-hd.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 $(CONVERT) $@

teapot-lines.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 --mode lines $(CONVERT) $@

teapot-points.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 --mode points $(CONVERT) $@

teapot-background.jpg: $(EXE)
	$(RUN) $(TEAPOT) 500 500 --mode lines --background "0.5 0.5 0.5" $(CONVERT) $@

teapot-solid.jpg: $(EXE)
	$(RUN) $(TEAPOT) 500 500 --shader solid --color "0 1 1" $(CONVERT) $@

bunny.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 $(CONVERT) $@

bunny-scale.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "50 50 50" $(CONVERT) $@

bunny-stretch.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "30 50 30" $(CONVERT) $@

bunny-translate.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "30 30 30" --translate '1 2 0' $(CONVERT) $@

bunny-rotate.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "50 50 50" --yaw 0.75 $(CONVERT) $@

$(EXE):
	cargo build --release

clean-output:
	rm -f *.jgr *.ps *.pdf *.jpg

clean: clean-output
	cargo clean
