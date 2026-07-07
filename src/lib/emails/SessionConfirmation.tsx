import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Hr,
  Html,
  Preview,
  Section,
  Tailwind,
  Text,
} from '@react-email/components';

interface Props {
  studentName: string;
  tutorName: string;
  courseTitle: string;
  scheduledAt: string; // formatted human string, e.g. "Fri, 15 Aug at 14:00 GMT+7"
  durationMin: number;
  joinUrl: string;
  manageUrl: string;
}

/**
 * Sent immediately after a session is booked. Keeps the copy tight — a
 * three-fact summary + one primary action (Join). The reminder pipeline fires
 * later at T-24h + T-1h with slightly different framing.
 */
export function SessionConfirmationEmail({
  studentName,
  tutorName,
  courseTitle,
  scheduledAt,
  durationMin,
  joinUrl,
  manageUrl,
}: Props) {
  return (
    <Html lang="en">
      <Head />
      <Preview>{`Your VieLang session with ${tutorName} is confirmed`}</Preview>
      <Tailwind>
        <Body className="bg-slate-50 font-sans">
          <Container className="mx-auto my-8 max-w-lg rounded-2xl bg-white p-8">
            <Heading className="mb-2 text-2xl font-bold text-slate-900">Session confirmed</Heading>
            <Text className="mb-6 text-sm text-slate-600">
              Hi {studentName}, your booking with {tutorName} is set. We'll email you a reminder 24
              hours and 1 hour before it starts.
            </Text>

            <Section className="mb-6 rounded-xl border border-slate-200 p-4">
              <Row label="Tutor" value={tutorName} />
              <Row label="Course" value={courseTitle} />
              <Row label="When" value={scheduledAt} />
              <Row label="Duration" value={`${durationMin} minutes`} />
            </Section>

            <Button
              href={joinUrl}
              className="rounded-lg bg-indigo-600 px-6 py-3 text-center text-sm font-semibold text-white"
            >
              Open the classroom
            </Button>
            <Text className="mt-2 text-xs text-slate-500">
              The classroom link becomes joinable 15 minutes before the start time.
            </Text>

            <Hr className="my-6 border-slate-200" />

            <Text className="text-xs text-slate-500">
              Need to reschedule or cancel? Head to{' '}
              <a href={manageUrl} className="text-indigo-600 underline">
                My sessions
              </a>{' '}
              at least 24 hours ahead.
            </Text>
            <Text className="mt-6 text-xs text-slate-400">VieLang · Learn English Online</Text>
          </Container>
        </Body>
      </Tailwind>
    </Html>
  );
}

function Row({ label, value }: { label: string; value: string }) {
  return (
    <div className="mb-2 flex items-center justify-between">
      <span className="text-xs font-semibold tracking-wider text-slate-500 uppercase">{label}</span>
      <span className="text-sm font-medium text-slate-800">{value}</span>
    </div>
  );
}
