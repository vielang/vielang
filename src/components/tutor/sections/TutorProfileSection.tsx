'use client';

import { useState } from 'react';
import { useRouter } from 'next/navigation';
import { toast } from 'sonner';
import { Loader2, Save } from 'lucide-react';
import { getAuthHeaders } from '@/contexts/AuthContext';
import { Button } from '@/components/ui/button';
import { Input } from '@/components/ui/input';
import { Label } from '@/components/ui/label';
import { Textarea } from '@/components/ui/textarea';
import type { User, TutorProfile } from '@/lib/types';
import { errorMessage } from '@/lib/errors';

interface Props {
  user: User;
  profile: TutorProfile;
}

const splitCsv = (s: string) =>
  s
    .split(',')
    .map((x) => x.trim())
    .filter(Boolean);

const joinCsv = (arr: string[]) => arr.join(', ');

export function TutorProfileSection({ user, profile }: Props) {
  const router = useRouter();
  const [saving, setSaving] = useState(false);

  const [form, setForm] = useState({
    name: user.name,
    bio: user.bio ?? '',
    avatar: user.avatar ?? '',
    hourly_rate_vnd: profile.hourly_rate_vnd,
    years_experience: profile.years_experience,
    intro_video_url: profile.intro_video_url ?? '',
    specialties: joinCsv(profile.specialties),
    certifications: joinCsv(profile.certifications),
    languages_spoken: joinCsv(profile.languages_spoken),
  });

  const submit = async (e: React.FormEvent) => {
    e.preventDefault();
    setSaving(true);
    try {
      const payload = {
        name: form.name.trim(),
        bio: form.bio.trim(),
        avatar: form.avatar.trim() || null,
        hourly_rate_vnd: Number(form.hourly_rate_vnd) || 0,
        years_experience: Number(form.years_experience) || 0,
        intro_video_url: form.intro_video_url.trim() || null,
        specialties: splitCsv(form.specialties),
        certifications: splitCsv(form.certifications),
        languages_spoken: splitCsv(form.languages_spoken),
      };
      const res = await fetch('/api/tutor/profile', {
        method: 'PATCH',
        headers: { 'Content-Type': 'application/json', ...(await getAuthHeaders()) },
        body: JSON.stringify(payload),
      });
      const data = await res.json();
      if (!res.ok) throw new Error(data?.error || 'save_failed');
      toast.success('Profile saved');
      router.refresh();
    } catch (err) {
      toast.error('Could not save profile', { description: errorMessage(err) });
    } finally {
      setSaving(false);
    }
  };

  return (
    <div className="space-y-6">
      <header>
        <h1 className="type-page text-slate-900 dark:text-slate-100">Profile</h1>
        <p className="mt-1 text-sm text-slate-500 dark:text-slate-400">
          This is what students see on your public page.
        </p>
      </header>

      <form
        onSubmit={submit}
        className="space-y-5 rounded-2xl border border-slate-200 bg-white p-5 shadow-sm dark:border-slate-800 dark:bg-slate-900"
      >
        <div className="grid grid-cols-1 gap-4 sm:grid-cols-2">
          <Field label="Display name" id="profile-name">
            <Input
              id="profile-name"
              value={form.name}
              onChange={(e) => setForm({ ...form, name: e.target.value })}
              maxLength={120}
              required
              disabled={saving}
            />
          </Field>
          <Field label="Avatar URL" id="profile-avatar" hint="Optional — leave blank for initials.">
            <Input
              id="profile-avatar"
              type="url"
              value={form.avatar}
              onChange={(e) => setForm({ ...form, avatar: e.target.value })}
              placeholder="https://…"
              disabled={saving}
            />
          </Field>
        </div>

        <Field label="Bio" id="profile-bio" hint="What should a first-time student know about you?">
          <Textarea
            id="profile-bio"
            value={form.bio}
            onChange={(e) => setForm({ ...form, bio: e.target.value })}
            rows={4}
            maxLength={2000}
            disabled={saving}
          />
          <p className="text-right text-[10px] text-slate-400 tabular-nums dark:text-slate-500">
            {form.bio.length}/2000
          </p>
        </Field>

        <div className="grid grid-cols-1 gap-4 sm:grid-cols-3">
          <Field label="Hourly rate (₫)" id="profile-rate">
            <Input
              id="profile-rate"
              type="number"
              min={0}
              step={10_000}
              value={form.hourly_rate_vnd}
              onChange={(e) => setForm({ ...form, hourly_rate_vnd: Number(e.target.value) })}
              disabled={saving}
            />
          </Field>
          <Field label="Years teaching" id="profile-exp">
            <Input
              id="profile-exp"
              type="number"
              min={0}
              max={80}
              value={form.years_experience}
              onChange={(e) => setForm({ ...form, years_experience: Number(e.target.value) })}
              disabled={saving}
            />
          </Field>
          <Field label="Intro video URL" id="profile-video" hint="YouTube / Vimeo / Loom.">
            <Input
              id="profile-video"
              type="url"
              value={form.intro_video_url}
              onChange={(e) => setForm({ ...form, intro_video_url: e.target.value })}
              placeholder="https://…"
              disabled={saving}
            />
          </Field>
        </div>

        <Field
          label="Specialties"
          id="profile-specialties"
          hint="Comma-separated. e.g. IELTS, Business, Kids"
        >
          <Input
            id="profile-specialties"
            value={form.specialties}
            onChange={(e) => setForm({ ...form, specialties: e.target.value })}
            disabled={saving}
          />
        </Field>

        <Field
          label="Certifications"
          id="profile-certs"
          hint="Comma-separated. e.g. TESOL, CELTA, IELTS 8.5"
        >
          <Input
            id="profile-certs"
            value={form.certifications}
            onChange={(e) => setForm({ ...form, certifications: e.target.value })}
            disabled={saving}
          />
        </Field>

        <Field label="Languages spoken" id="profile-langs" hint="2–6 char codes, e.g. en, vi, ko">
          <Input
            id="profile-langs"
            value={form.languages_spoken}
            onChange={(e) => setForm({ ...form, languages_spoken: e.target.value })}
            disabled={saving}
          />
        </Field>

        <div className="flex justify-end border-t border-slate-100 pt-3 dark:border-slate-800">
          <Button type="submit" disabled={saving} size="sm">
            {saving ? <Loader2 className="size-3.5 animate-spin" /> : <Save className="size-3.5" />}
            Save profile
          </Button>
        </div>
      </form>
    </div>
  );
}

function Field({
  label,
  hint,
  id,
  children,
}: {
  label: string;
  hint?: string;
  id?: string;
  children: React.ReactNode;
}) {
  return (
    <div className="space-y-1.5">
      <Label htmlFor={id} className="type-eyebrow">
        {label}
      </Label>
      {children}
      {hint && <p className="text-[10px] text-slate-400 dark:text-slate-500">{hint}</p>}
    </div>
  );
}
