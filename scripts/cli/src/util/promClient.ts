import promClient from "prom-client";
const { register } = promClient;

export const startCollection = () => {
	console.log(
		"Starting the collection of metrics, the metrics are available on /metrics"
	);
	promClient.collectDefaultMetrics();
};

export const injectMetricsRoute = (app: any) => {
	app.get("/metrics", async (_: any, res: any) => {
		res.set("Content-Type", register.contentType);
		res.end(await register.metrics());
	});
};

export const polyxBridgeLimitEquivocations = new promClient.Counter({
	name: "polymesh_offences_monitor_polyx_bridge_limit_per_transaction",
	help: "The number of polyx bridge limit equivocations and when.",
	labelNames: ["offenderAddress"],
});

export const polyxTotalBridgeLimitEquivocations = new promClient.Gauge({
	name: "polymesh_offences_monitor_total_polyx_bridge_limit_per_timelock_equivocations",
	help: "The number of total polyx bridge limit equivocations and when.",
	labelNames: ["offenderAddress"],
});
