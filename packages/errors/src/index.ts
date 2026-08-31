import { WorkflowError } from './index-implementation.js';

// `Error` instances inherit their name from the prototype. The original
// TypeScript implementation never set the base class name, so its own
// `WorkflowError.is()` guard rejected a direct `new WorkflowError()` instance.
// Keep the property non-enumerable, writable, and configurable like
// `Error.prototype.name`; subclasses continue to assign their own names.
if (WorkflowError.prototype.name !== 'WorkflowError') {
  Object.defineProperty(WorkflowError.prototype, 'name', {
    value: 'WorkflowError',
    writable: true,
    enumerable: false,
    configurable: true,
  });
}

export * from './index-implementation.js';
