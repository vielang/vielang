import {
  Body,
  Button,
  Container,
  Head,
  Heading,
  Html,
  Preview,
  Tailwind,
  Text,
} from '@react-email/components';

interface Props {
  studentName: string;
  tutorName: string;
  reviewUrl: string;
}

/**
 * Sent after a session's LiveKit `room_finished` webhook fires and status
 * flips to `completed`. Copy is intentionally short — long emails tank
 * review conversion.
 */
export function ReviewRequestEmail({ studentName, tutorName, reviewUrl }: Props) {
  return (
    <Html lang="en">
      <Head />
      <Preview>How was your VieLang session with {tutorName}?</Preview>
      <Tailwind>
        <Body className="bg-slate-50 font-sans">
          <Container className="mx-auto my-8 max-w-lg rounded-2xl bg-white p-8">
            <Heading className="mb-2 text-2xl font-bold text-slate-900">
              How was it, {studentName}?
            </Heading>
            <Text className="mb-6 text-sm text-slate-600">
              You just finished a lesson with {tutorName}. A quick review helps other students pick
              the right tutor — and helps {tutorName} keep getting better.
            </Text>
            <Button
              href={reviewUrl}
              className="rounded-lg bg-indigo-600 px-6 py-3 text-center text-sm font-semibold text-white"
            >
              Leave a review
            </Button>
            <Text className="mt-6 text-xs text-slate-400">VieLang · Learn English Online</Text>
          </Container>
        </Body>
      </Tailwind>
    </Html>
  );
}
