import adder from './functions/adder.as?raw';
import echo from './functions/echo.as?raw';
import fibonacci from './functions/fibonacci.as?raw';
import neverEnds from './functions/never-ends.as?raw';

// The functions Bob can start from. Each one is TypeScript-like source (AssemblyScript) that the page compiles to a
// WebAssembly module, so the person can change it before publishing. The adder and the Fibonacci function are ports of the
// Rust services in examples/wasm-services, and tests hold them to the same answers.

export interface FunctionInfo {
  id: string;
  label: string;
  serviceId: string;
  summary: string;
  /** What Alice's input starts as. */
  input: string;
  blurb: string;
  /** The source the editor starts with. */
  code: string;
  /** The Rust version this one is a port of, when there is one. */
  rust?: string;
}

const RUST = 'https://github.com/AlitheaBio/froglet/blob/main/examples/wasm-services';

export const FUNCTIONS: FunctionInfo[] = [
  {
    id: 'adder',
    label: 'Adder',
    serviceId: 'demo.adder',
    summary: 'Adds and multiplies two integers',
    input: '{"a": 6, "b": 7}',
    blurb: 'Reads two integers and returns their sum and product. It is a port of the Rust sample, and tests hold the two to the same answers.',
    code: adder,
    rust: `${RUST}/adder/src/lib.rs`,
  },
  {
    id: 'fibonacci',
    label: 'Fibonacci',
    serviceId: 'demo.fibonacci',
    summary: 'The nth Fibonacci number',
    input: '{"n": 10}',
    blurb: 'Returns the nth Fibonacci number, up to the 78th. It is a port of the Rust sample, and tests hold the two to the same answers.',
    code: fibonacci,
    rust: `${RUST}/fibonacci/src/lib.rs`,
  },
  {
    id: 'echo',
    label: 'Echo',
    serviceId: 'demo.echo',
    summary: 'Answers with its request',
    input: '{"hello": "world"}',
    blurb: 'The simplest function: it answers with its request. A good place to start from scratch.',
    code: echo,
  },
  {
    id: 'never-ends',
    label: 'Never ends',
    serviceId: 'demo.never-ends',
    summary: 'A function that never finishes',
    input: '{}',
    blurb: 'A function that loops forever. Bob stops it at his time limit and signs a receipt that says it failed. A real node stops it sooner, by counting fuel, which a browser cannot do. Delete the loop to see it answer.',
    code: neverEnds,
  },
];
