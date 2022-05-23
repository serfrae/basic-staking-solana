const CracoEsbuildPlugin = require("craco-esbuild");
const { ProvidePlugin } = require("webpack");

module.exports = {
	plugins: [{ plugin: CracoEsbuildPlugin }],
	style: {
		postcssOptions: {
			plugins: [require("tailwindcss"), require("autoprefixer")],
		},
	},
	typescript: {
		enableTypeChecking: true /* (default value)  */
	},
	webpack: {
		configure: (config) => {
			config.resolve.extensions.push(".wasm");

			config.module.rules.forEach((rule) => {
				(rule.oneOf || []).forEach((oneOf) => {
					if (oneOf.loader && oneOf.loader.indexOf("file-loader") >= 0) {
						oneOf.exclude.push(/\.wasm$/);
					}
				});
			});

			config.module.rules.push({
				test: /\.js$/,
				use: { loader: require.resolve("@open-wc/webpack-import-meta-loader") },
			});

			return config;
		},
		rules: [
			{
				test: /\.wasm$/,
				loader: "file-loader",
				type: "javascript/auto",
			},
			{
				test: /\.mjs$/,
				type: "javascript/auto",
			},
		],
		resolve: {
			mainFields: ["main"],
			extensions: [ '.ts', '.js' ],
			fallback: {
				"stream": require.resolve("stream-browserify"),
				"buffer": require.resolve("buffer")
			}
		},
		plugins: {
			add: [
				new ProvidePlugin({
					React: "react",
				}),
				new ProvidePlugin({
					Buffer: ['buffer', 'Buffer'],
				}),
				new ProvidePlugin({
					process: 'process/browser',
				}),
			],
		},
	},
};
