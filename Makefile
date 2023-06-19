flamegraph: flamegraph-apw flamegraph-pod

flamegraph-apw: flamegraph-apw.svg

flamegraph-pod: flamegraph-pod.svg

flamegraph-apw.svg: always
	mv flamegraph-apw.svg flamegraph-apw.old.svg
	cargo flamegraph -v --bench criterion --skip-after criterion::main -o $@ -- --bench --profile-time 5 apw
	rm -f perf.data perf.data.old

flamegraph-pod.svg: always
	mv flamegraph-pod.svg flamegraph-pod.old.svg
	cargo flamegraph -v --bench criterion --skip-after criterion::main -o $@ -- --bench --profile-time 5 pod
	rm -f perf.data perf.data.old

.PHONY: always
