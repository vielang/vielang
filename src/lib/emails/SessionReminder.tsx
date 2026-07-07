import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Html,
  Preview,
  Section,
  Tailwind,
  Text,
} from '@react-email/components';

interface Props {
  studentName: string;
  tutorName: string;
  scheduledAt: string;
  joinUrl: string;
  /** '24h' or '1h' — controls headline copy. */
  window: '24h' | '1h';
}

/**
 * T-24h and T-1h reminder. Same template, different framing selected by
 * `window`. Reduces duplication and keeps the visual language consistent.
 */
export function SessionReminderEmail({
  studentName,
  tutorName,
  scheduledAt,
  joinUrl,
  window,
}: Props) {
  const isNear = window === '1h';
  return (
    <Html lang="en">
      <Head />
      <Preview>
        {isNear
          ? `Your VieLang session starts in about an hour`
          : `Reminder: your VieLang session is tomorrow`}
      </Preview>
      <Tailwind>
        <Body className="bg-slate-50 font-sans">
          <Container className="mx-auto my-8 max-w-lg rounded-2xl bg-white p-8">
            <Heading className="mb-2 text-2xl font-bold text-slate-900">
              {isNear ? "It's almost time" : 'Your session is tomorrow'}
            </Heading>
            <Text className="mb-6 text-sm text-slate-600">
              Hi {studentName}, your lesson with {tutorName} is scheduled for {scheduledAt}. Grab a
              glass of water, find a quiet spot, and click below when you're ready.
            </Text>

            <Section className="mb-6">
              <Button
                href={joinUrl}
                className="rounded-lg bg-indigo-600 px-6 py-3 text-center text-sm font-semibold text-white"
              >
                {isNear ? 'Join now' : 'Preview the classroom'}
              </Button>
            </Section>

            <Text className="text-xs text-slate-500">
              {isNear
                ? 'The classroom is open. If Join fails, check that your camera + microphone are allowed.'
                : 'The Join button becomes active 15 minutes before the start time.'}
            </Text>
            <Text className="mt-6 text-xs text-slate-400">VieLang · Learn English Online</Text>
          </Container>
        </Body>
      </Tailwind>
    </Html>
  );
}
