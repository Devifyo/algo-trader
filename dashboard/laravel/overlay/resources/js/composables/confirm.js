// Promise-based confirmation dialog, replacing native window.confirm().
// Usage:
//   import { useConfirm } from '../composables/confirm';
//   const confirm = useConfirm();
//   if (!(await confirm('Halt all new entries now?', { tone: 'danger' }))) return;
// A single <ConfirmDialog /> (mounted in AppLayout) renders the shared state.
import { reactive } from 'vue';

export const confirmState = reactive({
    open: false,
    title: 'Please confirm',
    message: '',
    confirmText: 'Confirm',
    cancelText: 'Cancel',
    tone: 'warning', // 'warning' | 'danger' | 'default'
    _resolve: null,
});

export function useConfirm() {
    return (message, opts = {}) =>
        new Promise((resolve) => {
            confirmState.message = message;
            confirmState.title = opts.title ?? 'Please confirm';
            confirmState.confirmText = opts.confirmText ?? 'Confirm';
            confirmState.cancelText = opts.cancelText ?? 'Cancel';
            confirmState.tone = opts.tone ?? 'warning';
            confirmState._resolve = resolve;
            confirmState.open = true;
        });
}

// Called by ConfirmDialog when the user picks an option (or dismisses → false).
export function settleConfirm(value) {
    confirmState.open = false;
    const resolve = confirmState._resolve;
    confirmState._resolve = null;
    if (resolve) resolve(value);
}
