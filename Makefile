EXE=target/release/j-grangle
TEAPOT=obj/teapot.obj
BUNNY=obj/bunny.obj
RUN=cargo run --quiet --release --
CONVERT=| jgraph -P | convert -

all: 01-teapot.jpg 02-teapot-hd.jpg 03-teapot-lines.jpg 04-teapot-points.jpg 05-teapot-background.jpg 06-teapot-solid.jpg 07-bunny.jpg 08-bunny-scale.jpg 09-bunny-stretch.jpg 10-bunny-translate.jpg 11-bunny-rotate.jpg

01-teapot.jpg: $(EXE)
	$(RUN) $(TEAPOT) 100 100 $(CONVERT) $@

02-teapot-hd.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 $(CONVERT) $@

03-teapot-lines.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 --mode lines $(CONVERT) $@

04-teapot-points.jpg: $(EXE)
	$(RUN) $(TEAPOT) 1000 1000 --mode points $(CONVERT) $@

05-teapot-background.jpg: $(EXE)
	$(RUN) $(TEAPOT) 500 500 --mode lines --background "0.5 0.5 0.5" $(CONVERT) $@

06-teapot-solid.jpg: $(EXE)
	$(RUN) $(TEAPOT) 500 500 --shader solid --color "0 1 1" $(CONVERT) $@

07-bunny.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 $(CONVERT) $@

08-bunny-scale.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "50 50 50" $(CONVERT) $@

09-bunny-stretch.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "30 50 30" $(CONVERT) $@

10-bunny-translate.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "30 30 30" --translate '1 2 0' $(CONVERT) $@

11-bunny-rotate.jpg: $(EXE)
	$(RUN) $(BUNNY) 500 500 --scale "50 50 50" --yaw 0.75 $(CONVERT) $@

$(EXE):
	cargo build --release

clean-output:
	rm -f *.jgr *.ps *.pdf *.jpg

clean: clean-output
	cargo clean
