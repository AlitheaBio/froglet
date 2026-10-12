// @vitest-environment node
import { test } from 'vitest';
import assert from 'node:assert/strict';
import pageData from '../../data/live-services.json';
import {servicesView, recordedSessionAccess} from '../../data/live-services.mjs';
const pending = () => ({schema:pageData.schema,client_release:pageData.client_release,qualification:null,provider:null,access:{request_url:null,session_expires_at:null},capacity:{limits_state:'proposed',limits:{...pageData.capacity.limits},snapshot:null,public_status_endpoint:null},ingress:{state:'proposed',requests_per_second:1,burst:5,connections:2,measured_at:null,measurement_evidence_sha256:null},services:{arithmetic:{state:'pending',program:null,example:null},synthetic:{state:'pending',service_id:null,share_url:null,example:null}}});
const h = 'a'.repeat(64), at = '2026-10-05T12:00:00Z';
// Entirely invented fixtures for rendering policy; no runtime qualification.
function observed() {
  const s = pending();
  s.qualification = {passed:true, observed_at:at, root_evidence_sha256:h, independent_review_sha256:h};
  s.provider = {id:h, origin:'https://fixture.invalid', instance_count:1, metadata_url:'https://fixture.invalid/.well-known/agent-card.json'};
  s.access.session_expires_at = '2026-10-05T13:00:00Z';
  s.ingress = {state:'measured',requests_per_second:1,burst:5,connections:2,measured_at:at,measurement_evidence_sha256:h};
  s.capacity.limits_state = 'observed';
  s.capacity.snapshot = {observed_at:at,evidence_sha256:h,reserved_deals:7,issued_quotes:17,reserved_runtime_ms:90000};
  s.services.arithmetic = {state:'observed', program:{sha256:h,bytes:100,abi:'froglet.wasm.run_json.v1',download_url:'https://fixture.invalid/adder.wasm'}, example:{input:{a:5,b:8},expected_result:13,receipt_evidence_sha256:h}};
  s.services.synthetic = {state:'observed',service_id:'synthetic-terminology-demo',share_url:null,example:{input:{op:'select',collection:'rows',columns:['id','label'],equals:{id:'SYN:1'},limit:1},expected_result:{rows:[{id:'SYN:1',label:'Made-up label'}]},receipt_evidence_sha256:h}};
  return s;
}
test('pending data creates no endpoint, calls, capacity counts or availability claim', () => {const v=servicesView(pending());assert.equal(v.qualified,false);assert.equal(v.arithmetic,null);assert.equal(v.synthetic,null);assert.equal(v.remaining,null);});
test('observed data derives native MCP actions; arithmetic has no invented CLI command', () => {const v=servicesView(observed());assert.equal(JSON.parse(v.arithmetic.mcp).action,'run_compute');assert.equal(JSON.parse(v.synthetic.mcp).action,'invoke_service');assert.equal(v.arithmetic.cli,undefined);for(const e of [v.arithmetic,v.synthetic]) assert.equal(JSON.parse(e.mcp).max_price_sats,0);});
test('qualified provider requires exactly one instance, not a combined execution-slot count', () => {for(const count of [0,2,'1',null]) {const s=observed();s.provider.instance_count=count;assert.throws(()=>servicesView(s));}});
test('superseded worker-count fields cannot qualify a provider', () => {for(const kind of ['additional','replacement']) {const s=observed();s.provider.worker_count=1;if(kind==='replacement')delete s.provider.instance_count;assert.throws(()=>servicesView(s));}});
test('incomplete root or peer proof cannot enable observed cards', () => {for(const field of ['root_evidence_sha256','independent_review_sha256','observed_at']) {const s=observed();s.qualification[field]=null;assert.throws(()=>servicesView(s));}});
test('remaining count uses reserved maxima and retains actual observation time', () => {const v=servicesView(observed());assert.deepEqual(v.remaining,{deals:93,quotes:283,reserved_runtime_ms:110000,observed_at:at});});
test('remaining snapshot cannot be unlabeled, invented on pending state or noninteger', () => {for(const kind of ['timestamp','pending','counter']) {const s=observed();if(kind==='timestamp')s.capacity.snapshot.observed_at=null;if(kind==='pending')s.qualification=null;if(kind==='counter')s.capacity.snapshot.reserved_deals=true;assert.throws(()=>servicesView(s));}});
test('a depleted counter never creates negative remaining or replenishes capacity', () => {const s=observed();s.capacity.snapshot.reserved_runtime_ms=200001;assert.equal(servicesView(s).remaining.reserved_runtime_ms,0);});
test('unproved public capacity status path cannot become a live counter', () => {const s=observed();s.capacity.public_status_endpoint='https://fixture.invalid/status';assert.throws(()=>servicesView(s));});
test('qualified examples require actual finite capacity and recorded public ingress', () => {for(const kind of ['ingress','snapshot','limits','proof']) {const s=observed();if(kind==='ingress')s.ingress.state='proposed';if(kind==='snapshot')s.capacity.snapshot=null;if(kind==='limits')s.capacity.limits_state='proposed';if(kind==='proof')s.ingress.measurement_evidence_sha256=null;assert.throws(()=>servicesView(s));}});
test('private report or credential fields are rejected, including nested provider data', () => {for(const kind of ['root','provider']) {const s=observed();if(kind==='root')s.private_report={};else s.provider.token='never-publish';assert.throws(()=>servicesView(s));}});
test('metadata without a signed example does not produce a call', () => {const s=observed();s.services.synthetic.example.receipt_evidence_sha256=null;assert.throws(()=>servicesView(s));});
test('catalog rejects invented query/filter operations and compound equals', () => {for(const input of [{op:'query',collection:'rows'}, {op:'select',collection:'rows',filter:{}}, {op:'select',collection:'rows',equals:{id:[]}}]) {const s=observed();s.services.synthetic.example.input=input;assert.throws(()=>servicesView(s));}});
test('program must bind actual module, compiled ABI, size and download', () => {for(const change of [{sha256:null},{bytes:262145},{abi:'source-text'},{download_url:'javascript:bad'}]) {const s=observed();Object.assign(s.services.arithmetic.program,change);assert.throws(()=>servicesView(s));}});
test('CLI preserves arbitrary JSON as a safely quoted argument', () => {const s=observed();s.services.synthetic.example.input.equals.id="Alice's $(not_a_command)";const cli=servicesView(s).synthetic.cli;assert.ok(cli.includes("Alice'\"'\"'s $(not_a_command)"));assert.ok(cli.includes(' --max-price-sats 0 --json'));assert.ok(!cli.includes('run-compute'));});

test('invalid UTC calendar dates cannot label an observed service', () => {const s=observed();s.qualification.observed_at='2026-02-30T12:00:00Z';assert.throws(()=>servicesView(s));});

test('qualified sessions require an explicit later expiry', () => {for(const expires of [null,at,'2026-10-05T11:00:00Z']) {const s=observed();s.access.session_expires_at=expires;assert.throws(()=>servicesView(s));}});
test('pending data cannot retain invitation links or hidden observed examples', () => {for(const kind of ['access','program','service']) {const s=pending();if(kind==='access')s.access.request_url='https://fixture.invalid';if(kind==='program')s.services.arithmetic.program={};if(kind==='service')s.services.synthetic.service_id='stale';assert.throws(()=>servicesView(s));}});

test('metadata inspection and recorded examples survive session expiry without implying execution access', () => {const s=observed();const v=servicesView(s,Date.parse(s.access.session_expires_at));assert.equal(v.access_state,'recorded-session-expired');assert.ok(v.arithmetic);assert.ok(v.synthetic);assert.equal(recordedSessionAccess(s.access.session_expires_at,Date.parse(s.access.session_expires_at)-1),'invitation-required');});
test('metadata links require an actually selected same-origin public URL without credential query', () => {for(const metadata of [null,'https://other.invalid/card.json','https://fixture.invalid/card.json?token=private']) {const s=observed();s.provider.metadata_url=metadata;assert.throws(()=>servicesView(s));}});

test('configured ingress retains recorded timestamp and evidence requirements', () => {const s=observed();s.ingress.state='configured';assert.equal(servicesView(s).qualified,true);for(const field of ['measured_at','measurement_evidence_sha256']) {const broken=observed();broken.ingress.state='configured';broken.ingress[field]=null;assert.throws(()=>servicesView(broken));}});

test('public invitation request uses the approved Q&A discussion without granting access', () => {const url=new URL(pageData.access.request_url);assert.equal(url.origin,'https://github.com');assert.equal(url.pathname,'/AlitheaBio/froglet/discussions/new');assert.equal(url.searchParams.get('category'),'q-a');assert.equal(url.username,'');assert.equal(url.password,'');assert.equal(servicesView(pageData).qualified,true);});
