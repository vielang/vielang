'use client';

import { useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import Link from 'next/link';
import { toast } from 'sonner';
import { ExternalLink, Loader2, Search, ShieldCheck, ShieldOff } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { FilterPill } from '@/components/shared/FilterPill';
import { DataTable } from '@/components/ui/data-table';
import { Button, buttonVariants } from '@/components/ui/button';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import {
  Dialog,
  DialogContent,
  DialogDescription,
  DialogFooter,
  DialogHeader,
  DialogTitle,
} from '@/components/ui/dialog';
import { Textarea } from '@/components/ui/textarea';
import { initials } from '@/lib/format';
import type { User } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

type StatusFilter = 'all' | 'enabled' | 'disabled';

/**
 * TutorsSection — the master list of every account with role=tutor. Split
 * intentionally from PendingTutorsSection (which shows unapproved applicants);
 * this section is where admin bans, unbans, or clicks through to a tutor's
 * public profile.
 *
 * Enable/disable is the only mutation; approval flips over in the pending
 * queue. Role changes live in UsersSection to keep that one dialog focused.
 */
export function TutorsSection({ users, currentUserId }: { users: User[]; currentUserId: string }) {
  const router = useRouter();
  const tutors = useMemo(() => users.filter((u) => u.role === 'tutor'), [users]);

  const [status, setStatus] = useState<StatusFilter>('all');
  const [search, setSearch] = useState('');
  const [pending, setPending] = useState<string | null>(null);
  const [disableTarget, setDisableTarget] = useState<User | null>(null);
  const [reason, setReason] = useState('');

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return tutors.filter((t) => {
      if (status === 'enabled' && !t.enabled) return false;
      if (status === 'disabled' && t.enabled) return false;
      if (!needle) return true;
      return t.name.toLowerCase().includes(needle) || t.email.toLowerCase().includes(needle);
    });
  }, [tutors, status, search]);

  const toggleEnabled = async (u: User, next: boolean, disableReason?: string) => {
    setPending(u.id);
    try {
      const res = await fetch(`/api/admin/users/${u.id}/enabled`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ enabled: next, reason: disableReason ?? null }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'update_failed');
      toast.success(next ? 'Tutor re-enabled' : 'Tutor disabled');
      router.refresh();
    } catch (err) {
      toast.error('Could not update account', { description: errorMessage(err) });
    } finally {
      setPending(null);
      setDisableTarget(null);
      setReason('');
    }
  };

  const enabledCount = tutors.filter((t) => t.enabled).length;

  const columns = useMemo<ColumnDef<User>[]>(
    () => [
      {
        id: 'name',
        accessorKey: 'name',
        header: 'Tutor',
        cell: ({ row }) => (
          <div className="flex items-center gap-2.5">
            {row.original.avatar ? (
              <Image
                src={row.original.avatar}
                alt={row.original.name}
                width={32}
                height={32}
                className="size-8 rounded-full object-cover"
              />
            ) : (
              <div className="bg-brand flex size-8 items-center justify-center rounded-full text-xs font-semibold text-white">
                {initials(row.original.name)}
              </div>
            )}
            <span className="font-medium text-slate-800 dark:text-slate-100">
              {row.original.name}
            </span>
          </div>
        ),
      },
      {
        id: 'email',
        accessorKey: 'email',
        header: 'Email',
        meta: {
          cellClassName: 'max-w-[240px] truncate text-slate-500 dark:text-slate-400',
          hideBelow: 'lg',
        },
        cell: ({ row }) => <>{row.original.email}</>,
      },
      {
        id: 'status',
        accessorKey: 'enabled',
        header: 'Status',
        cell: ({ row }) => (
          <StatusPill enabled={row.original.enabled} reason={row.original.disabled_reason} />
        ),
      },
      {
        id: 'joined',
        accessorKey: 'created_at',
        header: 'Joined',
        sortDescFirst: true,
        meta: {
          cellClassName:
            'text-xs whitespace-nowrap text-slate-500 tabular-nums dark:text-slate-400',
          hideBelow: 'lg',
        },
        cell: ({ row }) => <>{new Date(row.original.created_at).toLocaleDateString()}</>,
      },
      {
        id: 'actions',
        header: 'Actions',
        enableSorting: false,
        meta: { headClassName: 'w-56' },
        cell: ({ row }) => (
          <RowActions
            tutor={row.original}
            pending={pending === row.original.id}
            isSelf={row.original.id === currentUserId}
            onDisable={() => setDisableTarget(row.original)}
            onEnable={() => toggleEnabled(row.original, true)}
          />
        ),
      },
    ],
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [currentUserId, pending],
  );

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Tutors</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          {enabledCount} active · {tutors.length - enabledCount} disabled ·{' '}
          <span className="tabular-nums">{tutors.length}</span> total tutor accounts.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search tutors by name or email…"
            className="h-9 pl-8"
          />
        </div>
        <div className="flex gap-1.5">
          {(['all', 'enabled', 'disabled'] as StatusFilter[]).map((s) => (
            <FilterPill
              key={s}
              active={status === s}
              onClick={() => setStatus(s)}
              label={s}
              size="md"
            />
          ))}
        </div>
      </div>

      <DataTable
        data={filtered}
        columns={columns}
        getRowId={(u) => u.id}
        emptyState={
          <div className="rounded-2xl border border-dashed border-slate-200 py-16 text-center dark:border-slate-800">
            <p className="text-sm text-slate-500 dark:text-slate-400">
              No tutors match this filter.
            </p>
          </div>
        }
        mobileCard={(row) => {
          const t = row.original;
          return (
            <div className="space-y-3 p-4">
              <div className="flex items-start gap-3">
                {t.avatar ? (
                  <Image
                    src={t.avatar}
                    alt={t.name}
                    width={40}
                    height={40}
                    className="size-10 shrink-0 rounded-full object-cover"
                  />
                ) : (
                  <div className="bg-brand flex size-10 shrink-0 items-center justify-center rounded-full text-sm font-semibold text-white">
                    {initials(t.name)}
                  </div>
                )}
                <div className="min-w-0 flex-1 space-y-0.5">
                  <p className="truncate font-medium text-slate-800 dark:text-slate-100">
                    {t.name}
                  </p>
                  <p className="truncate text-xs text-slate-500 dark:text-slate-400">{t.email}</p>
                  <p className="text-[10px] text-slate-400 dark:text-slate-500">
                    Joined {new Date(t.created_at).toLocaleDateString()}
                  </p>
                </div>
                <StatusPill enabled={t.enabled} reason={t.disabled_reason} compact />
              </div>
              <RowActions
                tutor={t}
                pending={pending === t.id}
                isSelf={t.id === currentUserId}
                onDisable={() => setDisableTarget(t)}
                onEnable={() => toggleEnabled(t, true)}
              />
            </div>
          );
        }}
      />

      {/* Disable dialog — requires a reason so the tutor sees a human message
          on the login page instead of a generic "account disabled". The bottom-
          sheet form-factor on mobile keeps the input near the thumb. */}
      <Dialog
        open={!!disableTarget}
        onOpenChange={(v) => {
          if (!v) {
            setDisableTarget(null);
            setReason('');
          }
        }}
      >
        <DialogContent className="sm:max-w-md">
          <DialogHeader>
            <DialogTitle>Disable {disableTarget?.name}?</DialogTitle>
            <DialogDescription>
              They will be signed out immediately and cannot log back in until re-enabled. Their
              existing sessions stay bookable to the students who already have them.
            </DialogDescription>
          </DialogHeader>
          <div className="space-y-2">
            <label
              htmlFor="disable-reason"
              className="text-xs font-semibold text-slate-500 dark:text-slate-400"
            >
              Reason (shown to the tutor on login)
            </label>
            <Textarea
              id="disable-reason"
              value={reason}
              onChange={(e) => setReason(e.target.value)}
              placeholder="e.g. Pending identity verification, please contact support."
              rows={3}
              maxLength={300}
            />
            <p className="text-[11px] text-slate-500 tabular-nums dark:text-slate-400">
              {reason.length}/300
            </p>
          </div>
          <DialogFooter>
            <Button
              variant="ghost"
              onClick={() => {
                setDisableTarget(null);
                setReason('');
              }}
              disabled={pending !== null}
            >
              Cancel
            </Button>
            <Button
              variant="destructive"
              onClick={() =>
                disableTarget && toggleEnabled(disableTarget, false, reason.trim() || undefined)
              }
              disabled={pending !== null}
            >
              {pending === disableTarget?.id ? (
                <Loader2 className="size-4 animate-spin" />
              ) : (
                <>
                  <ShieldOff className="size-4" /> Disable tutor
                </>
              )}
            </Button>
          </DialogFooter>
        </DialogContent>
      </Dialog>
    </div>
  );
}

function StatusPill({
  enabled,
  reason,
  compact,
}: {
  enabled: boolean;
  reason: string | null;
  compact?: boolean;
}) {
  const label = enabled ? 'active' : 'disabled';
  const tone = enabled
    ? 'bg-emerald-50 text-emerald-700 dark:bg-emerald-950/40 dark:text-emerald-300'
    : 'bg-red-50 text-red-700 dark:bg-red-950/40 dark:text-red-300';
  return (
    <div className={`inline-flex flex-col ${compact ? 'items-end' : 'items-start'}`}>
      <Badge
        className={cn(
          'rounded-full border-0 px-2 text-[10px] font-bold tracking-wide uppercase',
          tone,
        )}
      >
        {label}
      </Badge>
      {!enabled && reason && !compact && (
        <span
          title={reason}
          className="mt-1 max-w-[200px] truncate text-[10px] text-slate-500 italic dark:text-slate-400"
        >
          {reason}
        </span>
      )}
    </div>
  );
}

function RowActions({
  tutor,
  pending,
  isSelf,
  onDisable,
  onEnable,
}: {
  tutor: User;
  pending: boolean;
  isSelf: boolean;
  onDisable: () => void;
  onEnable: () => void;
}) {
  return (
    <div className="flex items-center gap-2">
      <Link
        href={`/tutors/${tutor.id}`}
        target="_blank"
        rel="noopener noreferrer"
        aria-label={`View ${tutor.name}'s public profile`}
        className={buttonVariants({ variant: 'outline', size: 'sm' })}
      >
        <ExternalLink className="size-3.5" />
        View
      </Link>
      {tutor.enabled ? (
        <Button
          variant="destructive"
          size="sm"
          onClick={onDisable}
          disabled={pending || isSelf}
          title={isSelf ? 'You cannot disable your own account' : undefined}
        >
          {pending ? (
            <Loader2 className="size-3.5 animate-spin" />
          ) : (
            <>
              <ShieldOff className="size-3.5" />
              Disable
            </>
          )}
        </Button>
      ) : (
        <Button size="sm" onClick={onEnable} disabled={pending}>
          {pending ? (
            <Loader2 className="size-3.5 animate-spin" />
          ) : (
            <>
              <ShieldCheck className="size-3.5" />
              Enable
            </>
          )}
        </Button>
      )}
    </div>
  );
}
