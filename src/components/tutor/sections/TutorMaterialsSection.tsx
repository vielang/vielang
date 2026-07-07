'use client';

import { useEffect, useState } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import {
  FileText,
  Video,
  Image as ImageIcon,
  Link2,
  Trash2,
  Plus,
  ExternalLink,
} from 'lucide-react';
import { Spinner } from '@/components/ui/spinner';
import { Input } from '@/components/ui/input';
import { Button } from '@/components/ui/button';
import {
  Form,
  FormControl,
  FormField,
  FormItem,
  FormLabel,
  FormMessage,
} from '@/components/ui/form';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { useDialogForm } from '@/hooks/use-dialog-form';
import { materialFormSchema } from '@/lib/schemas/forms';
import type { Course, Material, MaterialType } from '@/lib/types';

const TYPE_ICON: Record<MaterialType, React.ComponentType<{ className?: string }>> = {
  pdf: FileText,
  video: Video,
  image: ImageIcon,
  link: Link2,
};

const TYPE_OPTIONS: MaterialType[] = ['pdf', 'video', 'link', 'image'];

interface Props {
  courses: Course[];
}

export function TutorMaterialsSection({ courses }: Props) {
  const router = useRouter();
  const [courseId, setCourseId] = useState<string>(courses[0]?.id || '');
  const [items, setItems] = useState<Material[]>([]);
  const [loading, setLoading] = useState(false);
  const [busy, setBusy] = useState<string | null>(null);

  const form = useDialogForm({
    schema: materialFormSchema,
    defaultValues: { title: '', type: 'link' as MaterialType, url: '' },
    successTitle: 'Material added',
    errorTitle: 'Could not add material',
    onSubmit: async (values) => {
      if (!courseId) throw new Error('Pick a course first.');
      const res = await fetch(`/api/courses/${courseId}/materials`, {
        method: 'POST',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({
          title: values.title,
          type: values.type,
          url: values.url,
        }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'add_failed');
      setItems((prev) => [...prev, data.material as Material]);
      form.reset({ title: '', type: 'link', url: '' });
      router.refresh();
    },
  });

  useEffect(() => {
    if (!courseId) {
      setItems([]);
      return;
    }
    let cancelled = false;
    setLoading(true);
    (async () => {
      try {
        const res = await fetch(`/api/courses/${courseId}/materials`, {
          headers: await getAuthHeaders(),
        });
        if (!res.ok) throw new Error(`HTTP ${res.status}`);
        const data = await res.json();
        if (!cancelled) setItems(data.materials as Material[]);
      } catch (err: unknown) {
        if (!cancelled) {
          const message = err instanceof Error ? err.message : String(err);
          toast.error('Could not load materials', { description: message });
        }
      } finally {
        if (!cancelled) setLoading(false);
      }
    })();
    return () => {
      cancelled = true;
    };
  }, [courseId]);

  const remove = async (id: string) => {
    setBusy(id);
    try {
      const res = await fetch(`/api/materials/${id}`, {
        method: 'DELETE',
        headers: await getAuthHeaders(),
      });
      if (!res.ok) {
        const data = await res.json().catch(() => ({}));
        throw new Error(data?.error || 'delete_failed');
      }
      setItems((prev) => prev.filter((m) => m.id !== id));
      toast.success('Removed');
      router.refresh();
    } catch (err: unknown) {
      const message = err instanceof Error ? err.message : String(err);
      toast.error('Could not remove', { description: message });
    } finally {
      setBusy(null);
    }
  };

  if (courses.length === 0) {
    return (
      <div className="space-y-4">
        <header>
          <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
            Course materials
          </h1>
          <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
            You don't have any published courses to attach materials to yet.
          </p>
        </header>
      </div>
    );
  }

  return (
    <div className="space-y-6">
      <header>
        <h1 className="text-brand dark:text-accent-warm font-serif text-2xl font-bold">
          Course materials
        </h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          Attach worksheets, videos, links or images. Students see them once they've booked a
          session for that course.
        </p>
      </header>

      <label className="block max-w-md space-y-1">
        <span className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
          Course
        </span>
        <select
          value={courseId}
          onChange={(e) => setCourseId(e.target.value)}
          className="focus:border-brand/50 h-10 w-full rounded-lg border border-slate-200 bg-white px-3 text-sm focus:outline-none dark:border-slate-700 dark:bg-slate-900"
        >
          {courses.map((c) => (
            <option key={c.id} value={c.id}>
              {c.title_en}
            </option>
          ))}
        </select>
      </label>

      <Form {...form}>
        <form
          onSubmit={form.submit}
          className="rounded-2xl border border-slate-200 bg-white p-4 shadow-sm dark:border-slate-800 dark:bg-slate-900"
        >
          <div className="grid grid-cols-1 items-start gap-3 md:grid-cols-[1fr_140px_1fr_auto]">
            <FormField
              control={form.control}
              name="title"
              render={({ field }) => (
                <FormItem>
                  <FormLabel className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    Title
                  </FormLabel>
                  <FormControl>
                    <Input
                      {...field}
                      placeholder="e.g. Speaking Part 1 worksheet"
                      maxLength={200}
                      disabled={form.isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="type"
              render={({ field }) => (
                <FormItem>
                  <FormLabel className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    Type
                  </FormLabel>
                  <FormControl>
                    <select
                      value={field.value}
                      onChange={(e) => field.onChange(e.target.value as MaterialType)}
                      onBlur={field.onBlur}
                      disabled={form.isSubmitting}
                      className="focus:border-brand/50 h-11 w-full rounded-lg border border-slate-200 bg-white px-3 text-sm focus:outline-none disabled:opacity-50 dark:border-slate-700 dark:bg-slate-900"
                    >
                      {TYPE_OPTIONS.map((t) => (
                        <option key={t} value={t}>
                          {t}
                        </option>
                      ))}
                    </select>
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <FormField
              control={form.control}
              name="url"
              render={({ field }) => (
                <FormItem>
                  <FormLabel className="text-[10px] font-semibold tracking-wider text-slate-500 uppercase dark:text-slate-400">
                    URL
                  </FormLabel>
                  <FormControl>
                    <Input
                      type="url"
                      {...field}
                      placeholder="https://…"
                      disabled={form.isSubmitting}
                    />
                  </FormControl>
                  <FormMessage />
                </FormItem>
              )}
            />
            <div className="pt-[22px] md:pt-[22px]">
              <Button type="submit" disabled={form.isSubmitting} className="w-full md:w-auto">
                {form.isSubmitting ? (
                  <Spinner className="size-3.5" />
                ) : (
                  <Plus className="size-3.5" />
                )}
                Add
              </Button>
            </div>
          </div>
        </form>
      </Form>

      <div className="rounded-2xl border border-slate-200 bg-white shadow-sm dark:border-slate-800 dark:bg-slate-900">
        {loading ? (
          <div className="flex items-center justify-center py-10 text-slate-400">
            <Spinner className="size-5" />
          </div>
        ) : items.length === 0 ? (
          <div className="py-12 text-center">
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No materials for this course yet.
            </p>
          </div>
        ) : (
          <ul className="divide-y divide-slate-100 dark:divide-slate-800">
            {items.map((m) => {
              const Icon = TYPE_ICON[m.type];
              return (
                <li key={m.id} className="flex items-center gap-3 p-4">
                  <div className="bg-brand/10 text-brand flex size-10 shrink-0 items-center justify-center rounded-lg">
                    <Icon className="size-4" />
                  </div>
                  <div className="min-w-0 flex-1">
                    <p className="truncate text-sm font-semibold text-slate-900 dark:text-slate-100">
                      {m.title}
                    </p>
                    <a
                      href={m.url}
                      target="_blank"
                      rel="noopener noreferrer"
                      className="hover:text-brand inline-flex items-center gap-1 text-[11px] text-slate-500 dark:text-slate-400"
                    >
                      <span className="font-semibold tracking-wide uppercase">{m.type}</span>
                      <ExternalLink className="size-3" />
                    </a>
                  </div>
                  <button
                    type="button"
                    onClick={() => remove(m.id)}
                    disabled={busy === m.id}
                    className="inline-flex size-8 items-center justify-center rounded-md text-slate-400 hover:bg-red-50 hover:text-red-600 disabled:opacity-40 dark:hover:bg-red-950/40"
                    aria-label="Delete material"
                  >
                    {busy === m.id ? (
                      <Spinner className="size-3.5" />
                    ) : (
                      <Trash2 className="size-3.5" />
                    )}
                  </button>
                </li>
              );
            })}
          </ul>
        )}
      </div>
    </div>
  );
}
