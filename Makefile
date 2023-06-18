flamegraph: flamegraph-apw flamegraph-pod

flamegraph-apw: flamegraph-apw.svg

flamegraph-pod: flamegraph-pod.svg

flamegraph-apw.svg: always
	cargo flamegraph -v --bench criterion --skip-after criterion::main -- --bench --profile-time 5 apw
	rm -f perf.data perf.data.old
	mv flamegraph-apw.svg flamegraph-apw.old.svg
	mv flamegraph.svg flamegraph-apw.svg

flamegraph-pod.svg: always
	cargo flamegraph -v --bench criterion --skip-after criterion::main -- --bench --profile-time 5 pod
	rm -f perf.data perf.data.old
	mv flamegraph-pod.svg flamegraph-pod.old.svg
	mv flamegraph.svg flamegraph-pod.svg

.PHONY: always
