'use client';

import { useCallback } from 'react';
import { useForm, type DefaultValues, type FieldValues, type UseFormReturn } from 'react-hook-form';
import { zodResolver } from '@hookform/resolvers/zod';
import { toast } from 'sonner';
import type { ZodType } from 'zod';

// Shared plumbing for every dialog form on the site: zod-validated
// react-hook-form + toast-on-success/error + one `submit` handler you drop into
// <form onSubmit>.
//
// Callers own the fields (rendered with shadcn <FormField>) and the mutation
// itself. This hook standardizes what happens around it — parse the form
// values with zod, run the mutation, toast the outcome, keep RHF's submitting
// flag in sync so the footer can disable itself.

interface UseDialogFormOpts<TValues extends FieldValues> {
  // ZodType output is inferred as TValues by the caller; input side is not
  // typed strictly so callers with optional/default fields still fit.
  schema: ZodType<TValues, unknown>;
  defaultValues: DefaultValues<TValues>;
  /**
   * Called with the parsed values on submit.
   *
   * - Resolve → success toast fires.
   * - Return `'handled'` → hook stays silent (caller already toasted, usually
   *   because the failure needs specific UI copy + retry, e.g. slot_taken).
   * - Throw → error toast with the thrown message as description.
   */
  onSubmit: (values: TValues) => Promise<void | 'handled'> | void | 'handled';
  successTitle: string;
  successDescription?: string;
  errorTitle: string;
}

export interface DialogFormHandle<TValues extends FieldValues> extends UseFormReturn<TValues> {
  submit: (e?: React.BaseSyntheticEvent) => Promise<void>;
  isSubmitting: boolean;
}

export function useDialogForm<TValues extends FieldValues>({
  schema,
  defaultValues,
  onSubmit,
  successTitle,
  successDescription,
  errorTitle,
}: UseDialogFormOpts<TValues>): DialogFormHandle<TValues> {
  // The zod v4 resolver's generics don't line up with RHF's TValues without a
  // hop through unknown. Runtime behavior is correct: RHF passes the parsed
  // z.infer<schema> to onSubmit; the cast just calms the type-level check.
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  const resolver = zodResolver(schema as unknown as ZodType<any, any>) as any;
  const form = useForm<TValues>({
    resolver,
    defaultValues,
    mode: 'onTouched',
  });

  const submit = useCallback(
    async (e?: React.BaseSyntheticEvent) => {
      await form.handleSubmit(async (values) => {
        try {
          const result = await onSubmit(values);
          if (result === 'handled') return;
          toast.success(
            successTitle,
            successDescription ? { description: successDescription } : undefined,
          );
        } catch (err) {
          const message = err instanceof Error ? err.message : String(err);
          toast.error(errorTitle, { description: message });
        }
      })(e);
    },
    [form, onSubmit, successTitle, successDescription, errorTitle],
  );

  return {
    ...form,
    submit,
    isSubmitting: form.formState.isSubmitting,
  };
}
