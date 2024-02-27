flamegraph: always
	cargo flamegraph --profile bench --package=$(PACKAGE) --output=flamegraph.svg -- "$(MOD)"
	rm -f perf.data perf.data.old

flamegraph-single: always
	RAYON_NUM_THREADS=1 cargo flamegraph --profile bench --package=$(PACKAGE) --output=flamegraph-single.svg -- "$(MOD)"
	rm -f perf.data perf.data.old

capnp: src/capnp/fileheader_capnp.rs src/capnp/pdxfile_capnp.rs

src/capnp/fileheader_capnp.rs: capnp/fileheader.capnp
	capnpc -orust:src $<

src/capnp/pdxfile_capnp.rs: capnp/pdxfile.capnp
	capnpc -orust:src $<

.PHONY: always capnp
