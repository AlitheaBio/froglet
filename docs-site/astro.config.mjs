// @ts-check
import { defineConfig } from 'astro/config';
import starlight from '@astrojs/starlight';
import remarkMath from 'remark-math';
import rehypeMathjax from 'rehype-mathjax';

const site = process.env.SITE_URL ?? 'https://froglet.dev';

export default defineConfig({
	site,
	markdown: {
		remarkPlugins: [remarkMath],
		rehypePlugins: [rehypeMathjax],
	},
	integrations: [
			starlight({
				title: 'Froglet',
				description: 'Identity. Execution. Settlement.',
				disable404Route: true,
				social: [{ icon: 'github', label: 'GitHub', href: 'https://github.com/AlitheaBio/froglet' }],
			head: [
				{ tag: 'link', attrs: { rel: 'icon', href: '/favicon.svg', type: 'image/svg+xml' } },
			],
			customCss: ['./src/styles/custom.css'],
			components: {
				ThemeProvider: './src/components/ThemeProvider.astro',
				Head: './src/components/StarlightHead.astro',
				Header: './src/components/StarlightHeader.astro',
				Footer: './src/components/StarlightFooter.astro',
				Sidebar: './src/components/StarlightSidebar.astro',
			},
				sidebar: [
					{
						label: 'Start Here',
						items: [
							{ label: 'Docs Home', slug: 'docs' },
							{ label: 'What Is Froglet?', slug: 'learn/introduction' },
							{ label: 'Share Data or a Tool', slug: 'learn/share-services' },
							{ label: 'Run Locally', slug: 'learn/quickstart' },
							{ label: 'Connect Agents', slug: 'learn/agents' },
							{ label: 'MCP & A2A Compute', slug: 'learn/agent-interoperability' },
							{ label: 'Hosted Trial Status', slug: 'learn/cloud-trial' },
						],
					},
					{
						label: 'Concepts',
						items: [
							{ label: 'Protocol Walkthrough', slug: 'demo' },
							{ label: 'Identity', slug: 'learn/identity' },
							{ label: 'Deal Flow', slug: 'learn/deal-flow' },
							{ label: 'Settlement', slug: 'learn/settlement' },
							{ label: 'Trust & Economics', slug: 'learn/economics' },
							{ label: 'How Froglet Compares', slug: 'learn/comparison' },
						],
					},
					{
						label: 'Reference',
						items: [
							{ label: 'Provider Setup', slug: 'learn/provider-onboarding' },
							{ label: 'LLM Install Contract', slug: 'learn/llm-self-install' },
							{ label: 'Plugin Distribution', slug: 'learn/plugin-distribution' },
							{ label: 'Payment Rails', slug: 'learn/payment-rails' },
							{ label: 'Marketplace', slug: 'marketplace/overview' },
							{ label: 'Kernel', slug: 'spec/kernel' },
							{ label: 'Conformance', slug: 'spec/conformance' },
							{ label: 'Service Binding', slug: 'spec/service-binding' },
							{ label: 'Architecture', slug: 'architecture/overview' },
							{ label: 'Crate Structure', slug: 'architecture/crates' },
							{ label: 'File Reference', slug: 'architecture/files' },
						],
					},
			],
		}),
	],
});
