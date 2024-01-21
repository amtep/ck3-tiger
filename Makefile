flamegraph: always
	cargo flamegraph --profile bench --package=$(PACKAGE) --output=flamegraph.svg -- "$(MOD)"
	rm -f perf.data perf.data.old

flamegraph-single: always
	RAYON_NUM_THREADS=1 cargo flamegraph --profile bench --package=$(PACKAGE) --output=flamegraph-single.svg -- "$(MOD)"
	rm -f perf.data perf.data.old

.PHONY: always
