GRAPHDIR=./graphs
DOTFILES=$(wildcard $(GRAPHDIR)/*.dot)
GMLFILES=$(patsubst %.dot, %.gml, $(DOTFILES))

graphdir:
	mkdir -p $(GRAPHDIR)

$(GRAPHDIR)/%.gml: $(GRAPHDIR)/%.dot graphdir 
	gv2gml $< > $@

graphs: graphdir $(GMLFILES)

clean:
	rm -rf $(GRAPHDIR)

.PHONY: graphs
