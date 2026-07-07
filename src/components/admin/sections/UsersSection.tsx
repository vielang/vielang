'use client';

import { useMemo, useState } from 'react';
import { useRouter } from 'next/navigation';
import Image from 'next/image';
import { toast } from 'sonner';
import { Loader2, Search } from 'lucide-react';
import type { ColumnDef } from '@tanstack/react-table';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { FilterPill } from '@/components/shared/FilterPill';
import { DataTable } from '@/components/ui/data-table';
import { Badge } from '@/components/ui/badge';
import { Input } from '@/components/ui/input';
import { cn } from '@/lib/utils';
import {
  Select,
  SelectContent,
  SelectItem,
  SelectTrigger,
  SelectValue,
} from '@/components/ui/select';
import { initials } from '@/lib/format';
import type { User } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

const ROLES: (User['role'] | 'all')[] = ['all', 'user', 'tutor', 'admin'];

const ROLE_TONE: Record<User['role'], string> = {
  admin: 'bg-amber-50 dark:bg-amber-950/40 text-amber-700 dark:text-amber-300',
  tutor: 'bg-sky-50 dark:bg-sky-950/40 text-sky-700 dark:text-sky-300',
  user: 'bg-emerald-50 dark:bg-emerald-950/40 text-emerald-700 dark:text-emerald-300',
};

function RolePill({ role }: { role: User['role'] }) {
  return (
    <Badge
      className={cn(
        'rounded-full border-0 px-2 text-[10px] font-bold tracking-wide uppercase',
        ROLE_TONE[role],
      )}
    >
      {role}
    </Badge>
  );
}

function UserAvatar({ user, size = 32 }: { user: User; size?: 32 | 40 }) {
  const px = size === 40 ? 'size-10' : 'size-8';
  const text = size === 40 ? 'text-sm' : 'text-xs';
  if (user.avatar) {
    return (
      <Image
        src={user.avatar}
        alt={user.name}
        width={size}
        height={size}
        className={`${px} shrink-0 rounded-full object-cover`}
      />
    );
  }
  return (
    <div
      className={`bg-brand flex ${px} shrink-0 items-center justify-center rounded-full ${text} font-semibold text-white`}
    >
      {initials(user.name)}
    </div>
  );
}

/** Role picker — shadcn Select. Disables demoting self via `disabled` on the
 *  user + tutor items when the row belongs to the signed-in admin. */
function RoleSelect({
  user,
  isSelf,
  pending,
  onChange,
  variant = 'compact',
}: {
  user: User;
  isSelf: boolean;
  pending: boolean;
  onChange: (next: User['role']) => void;
  variant?: 'compact' | 'full';
}) {
  return (
    <Select value={user.role} onValueChange={(v) => onChange(v as User['role'])} disabled={pending}>
      <SelectTrigger
        aria-label={`Change role for ${user.name}`}
        className={variant === 'full' ? 'h-11 w-full text-sm' : 'h-8 w-32 text-xs'}
      >
        <SelectValue />
      </SelectTrigger>
      <SelectContent>
        {/* You can't demote yourself — the server also blocks this but greying
            it out avoids a confusing round-trip. */}
        <SelectItem value="user" disabled={isSelf}>
          user
        </SelectItem>
        <SelectItem value="tutor" disabled={isSelf}>
          tutor
        </SelectItem>
        <SelectItem value="admin">admin</SelectItem>
      </SelectContent>
    </Select>
  );
}

export function UsersSection({ users, currentUserId }: { users: User[]; currentUserId: string }) {
  const router = useRouter();
  const [role, setRole] = useState<User['role'] | 'all'>('all');
  const [search, setSearch] = useState('');
  const [pending, setPending] = useState<string | null>(null);

  const filtered = useMemo(() => {
    const needle = search.trim().toLowerCase();
    return users.filter((u) => {
      if (role !== 'all' && u.role !== role) return false;
      if (!needle) return true;
      return u.name.toLowerCase().includes(needle) || u.email.toLowerCase().includes(needle);
    });
  }, [users, role, search]);

  const changeRole = async (id: string, next: User['role']) => {
    setPending(id);
    try {
      const res = await fetch(`/api/admin/users/${id}/role`, {
        method: 'PUT',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify({ role: next }),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'update_failed');
      toast.success(`Role updated to ${next}`);
      router.refresh();
    } catch (err) {
      toast.error('Could not update role', { description: errorMessage(err) });
    } finally {
      setPending(null);
    }
  };

  const columns = useMemo<ColumnDef<User>[]>(
    () => [
      {
        id: 'name',
        accessorKey: 'name',
        header: 'Name',
        cell: ({ row }) => (
          <div className="flex items-center gap-2.5">
            <UserAvatar user={row.original} />
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
        meta: { cellClassName: 'max-w-[240px] truncate text-slate-500 dark:text-slate-400' },
        cell: ({ row }) => <>{row.original.email}</>,
      },
      {
        id: 'role',
        accessorKey: 'role',
        header: 'Role',
        cell: ({ row }) => <RolePill role={row.original.role} />,
      },
      {
        id: 'change',
        header: 'Change role',
        enableSorting: false,
        meta: { headClassName: 'w-40' },
        cell: ({ row }) => {
          const u = row.original;
          return (
            <div className="flex items-center gap-1.5">
              <RoleSelect
                user={u}
                isSelf={u.id === currentUserId}
                pending={pending === u.id}
                onChange={(next) => changeRole(u.id, next)}
              />
              {pending === u.id && <Loader2 className="size-3.5 animate-spin text-slate-400" />}
              {u.id === currentUserId && (
                <span className="text-[10px] whitespace-nowrap text-slate-400 dark:text-slate-500">
                  (you)
                </span>
              )}
            </div>
          );
        },
      },
    ],
    // changeRole + pending are stable closures over React state — TanStack
    // reads columns fresh every render so no memo invalidation needed.
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [currentUserId, pending],
  );

  return (
    <div className="space-y-4">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Users</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          {filtered.length} of {users.length} accounts.
        </p>
      </header>

      <div className="flex flex-wrap gap-2">
        <div className="relative min-w-[200px] flex-1">
          <Search className="text-muted-foreground pointer-events-none absolute top-1/2 left-2.5 size-3.5 -translate-y-1/2" />
          <Input
            value={search}
            onChange={(e) => setSearch(e.target.value)}
            placeholder="Search by name or email…"
            className="h-9 pl-8"
          />
        </div>
        <div className="flex gap-1.5">
          {ROLES.map((r) => (
            <FilterPill
              key={r}
              active={role === r}
              onClick={() => setRole(r)}
              label={r}
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
              No users match this filter.
            </p>
          </div>
        }
        mobileCard={(row) => {
          const u = row.original;
          return (
            <div className="space-y-3 p-4">
              <div className="flex items-start gap-3">
                <UserAvatar user={u} size={40} />
                <div className="min-w-0 flex-1">
                  <p className="truncate font-medium text-slate-800 dark:text-slate-100">
                    {u.name}
                    {u.id === currentUserId && (
                      <span className="ml-1 text-[10px] text-slate-400 dark:text-slate-500">
                        (you)
                      </span>
                    )}
                  </p>
                  <p className="truncate text-xs text-slate-500 dark:text-slate-400">{u.email}</p>
                </div>
                <RolePill role={u.role} />
              </div>
              <div className="flex items-center gap-2">
                <span className="type-eyebrow shrink-0 text-slate-500">Role</span>
                <RoleSelect
                  user={u}
                  isSelf={u.id === currentUserId}
                  pending={pending === u.id}
                  onChange={(next) => changeRole(u.id, next)}
                  variant="full"
                />
                {pending === u.id && <Loader2 className="size-4 animate-spin text-slate-400" />}
              </div>
            </div>
          );
        }}
      />
    </div>
  );
}
