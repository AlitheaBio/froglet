export interface AgentRouteMetadata {
	route: string;
	page_role: string;
	agent_purpose: string;
	primary_task: string;
	relevant_tasks: string[];
	evidence_boundary: string;
	canonical_sources: string[];
	preferred_next_action: string;
}

export const routeAgentMetadata = {
	home: {
		route: '/',
		page_role: 'entrypoint',
		agent_purpose: 'Prepare a useful selected service with separate installation and publication approvals.',
		primary_task: 'publish-service',
		relevant_tasks: ['publish-service', 'consume-service', 'bounded-compute', 'hosted-proof', 'receipt-artifact-verify'],
		evidence_boundary: 'Public beta; stable qualification remains open. Local evidence does not prove public activation or recipient execution.',
		canonical_sources: ['/llms.txt', '/agent-tasks.json', '/learn/cloud-trial/'],
		preferred_next_action: 'Read /publish/agent.md, inspect the source, and show a selected preview before public approval.',
	},
	publish: {
        route: '/publish/',
        page_role: 'service_publishing',
        agent_purpose: 'Prepare selected catalogs or Wasm functions, verify locally, and obtain exact public approval.',
        primary_task: 'publish-service',
        relevant_tasks: ['publish-service', 'local-install-proposal'],
        evidence_boundary: 'Public beta; stable qualification remains open. Installation, local execution, public reachability, marketplace activation, and recipient execution are separate evidence.',
        canonical_sources: ['/publish/agent.md', '/agent-tasks.json', '/install/agent.md'],
        preferred_next_action: 'Inspect the source and propose the smallest useful selection. Preserve existing files and settings.',
    },
    service: {
        route: '/service/',
        page_role: 'shared_service',
        agent_purpose: 'Consume the identity-bound service in this link using the same native integration.',
        primary_task: 'consume-service',
        relevant_tasks: ['consume-service', 'local-install-proposal', 'receipt-artifact-verify'],
        evidence_boundary: 'Page availability is a recent observation; the requester must verify the signed offer and its own execution receipt.',
        canonical_sources: ['/learn/share-services/', '/agent-tasks.json'],
        preferred_next_action: 'Check current provider metadata, use free-only native invocation, and report receipt verification.',
    },
	marketplace: {
		route: '/marketplace/',
		page_role: 'marketplace_snapshot',
		agent_purpose: 'Inspect public marketplace counts and copy an evidence snapshot without overstating hosted paid rails.',
		primary_task: 'marketplace-evidence',
		relevant_tasks: ['marketplace-evidence', 'receipt-feed-check', 'local-install-proposal'],
		evidence_boundary: 'Marketplace counts are observed public index data, not proof that hosted paid rails or mainnet money are live.',
		canonical_sources: ['/marketplace/', '/agent-tasks.json', 'https://marketplace.froglet.dev/v1/providers?limit=12', 'https://marketplace.froglet.dev/v1/offers?limit=24'],
		preferred_next_action: 'Use the copy evidence action, then cite snapshot time, providers, offers, receipts, and source links.',
	},
	demo: {
		route: '/demo/',
		page_role: 'interactive_walkthrough',
		agent_purpose: 'Use the browser walkthrough only as conceptual orientation; use hosted-proof for live evidence.',
		primary_task: 'hosted-proof',
		relevant_tasks: ['hosted-proof', 'local-install-proposal'],
		evidence_boundary: 'The interactive demo is educational and does not replace live hosted or local verification.',
		canonical_sources: ['/demo/', '/llms.txt', '/agent-tasks.json'],
		preferred_next_action: 'After orientation, run hosted-proof instead of summarizing the walkthrough as proof.',
	},
	managed: {
		route: '/managed/',
		page_role: 'organization_vision_and_current_controls',
		agent_purpose: 'Distinguish what Froglet applies today (scope, access, limits, withdrawal, evidence) from planned contract automation and managed hosting, and plan a small non-confidential pilot.',
		primary_task: 'local-install-proposal',
		relevant_tasks: ['local-install-proposal', 'hosted-proof'],
		evidence_boundary: 'Public beta. Services are open until access mode and finite allowances are configured. Current invitations are provider-scoped and require technical setup and deployed verification. The guided relay path exposes requests and results to the relay operator, and signed receipts are public. Managed hosting, arbitrary contract automation, SLA management, and organization-wide policy are not available.',
		canonical_sources: ['/managed/', '/learn/share-services/', '/learn/quickstart/', '/agent-tasks.json'],
		preferred_next_action: 'Choose one small non-confidential exchange and evaluate supported self-hosted controls. Do not promise managed hosting or automatic enforcement of an agreement.',
	},
	openSource: {
		route: '/open-source/',
		page_role: 'repository_orientation',
		agent_purpose: 'Orient agents toward the kernel, bounded execution runtime, native MCP, optional configured A2A, and conformance boundaries.',
		primary_task: 'local-install-proposal',
		relevant_tasks: ['local-install-proposal', 'bounded-compute', 'receipt-artifact-verify'],
		evidence_boundary: 'Current source supports free local MCP/A2A jobs for configured counterparties. Source tests do not prove installed-release support, actual agent attachment, public operation, real-money settlement or output correctness.',
		canonical_sources: ['/open-source/', '/learn/agent-interoperability/', 'https://github.com/AlitheaBio/froglet', 'https://github.com/AlitheaBio/froglet/blob/main/conformance/README.md', '/agent-tasks.json'],
		preferred_next_action: 'Run cargo run -p froglet-verify -- conformance/kernel_v1.json to confirm the frozen vectors verify offline, then choose the smallest local verification path for the user context.',
	},
	verifyReceipt: {
		route: '/verify-receipt/',
		page_role: 'receipt_local_verifier',
		agent_purpose: 'Verify supplied signed artifacts and chains locally with explicit verification limits.',
		primary_task: 'receipt-artifact-verify',
		relevant_tasks: ['receipt-artifact-verify', 'receipt-feed-check'],
		evidence_boundary: 'Checks signatures, canonical hashes, artifact semantics, and supplied chain links locally using WASM. Does not prove external settlement, real-world identity, truthful execution, or current expiry.',
		canonical_sources: ['/verify-receipt/', '/agent-tasks.json', '/spec/kernel/'],
		preferred_next_action: 'Paste the artifact JSON, run Verify evidence, and report the artifact and chain results including anything not evaluated.',
	},
	privacy: {
		route: '/privacy/',
		page_role: 'privacy_boundary',
		agent_purpose: 'Explain public website, hosted proof, and local/self-hosted privacy boundaries without inferring unobserved behavior.',
		primary_task: 'chat-only-fallback',
		relevant_tasks: ['chat-only-fallback', 'hosted-proof', 'local-install-proposal'],
		evidence_boundary: 'Privacy claims are policy and architecture boundaries unless verified against live routes or local configuration.',
		canonical_sources: ['/privacy/', '/llms.txt', '/agent-tasks.json'],
		preferred_next_action: 'Use this page to bound privacy claims, then run hosted-proof or local checks for runtime evidence.',
	},
	terms: {
		route: '/terms/',
		page_role: 'usage_policy',
		agent_purpose: 'State the terms of service, acceptable use policy, and takedown process for the first-party hosted Froglet services.',
		primary_task: 'chat-only-fallback',
		relevant_tasks: ['chat-only-fallback', 'hosted-proof'],
		evidence_boundary: 'Terms apply to the hosted services only; self-hosted nodes are governed by the Apache-2.0 license, not this page.',
		canonical_sources: ['/terms/', '/privacy/', '/llms.txt'],
		preferred_next_action: 'Use this page to bound usage and takedown claims; file marketplace complaints via the arbiter complaint endpoint.',
	},
} as const satisfies Record<string, AgentRouteMetadata>;
