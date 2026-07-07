-- Fix refresh_tutor_rating() to handle DELETE.
--
-- The original version referenced NEW.tutor_id, which is NULL on DELETE,
-- so removing a review left tutor_profiles.rating_avg / session_count
-- stale (e.g. still 5.0 with 0 reviews). Use COALESCE(NEW.tutor_id,
-- OLD.tutor_id) so the same statement works for INSERT / UPDATE / DELETE.

CREATE OR REPLACE FUNCTION refresh_tutor_rating() RETURNS TRIGGER AS $$
DECLARE
  affected_tutor UUID := COALESCE(NEW.tutor_id, OLD.tutor_id);
BEGIN
  UPDATE tutor_profiles tp
     SET rating_avg = COALESCE((
           SELECT ROUND(AVG(rating)::numeric, 1)
             FROM reviews r
            WHERE r.tutor_id = tp.user_id
         ), 0),
         session_count = COALESCE((
           SELECT COUNT(*) FROM sessions s
            WHERE s.tutor_id = tp.user_id AND s.status = 'completed'
         ), 0)
   WHERE tp.user_id = affected_tutor;
  RETURN COALESCE(NEW, OLD);
END;
$$ LANGUAGE plpgsql;

-- Recompute all tutor_profiles once so stale rows from the buggy version
-- get healed. Fast at our scale.
UPDATE tutor_profiles tp
   SET rating_avg = COALESCE((
         SELECT ROUND(AVG(rating)::numeric, 1)
           FROM reviews r
          WHERE r.tutor_id = tp.user_id
       ), 0),
       session_count = COALESCE((
         SELECT COUNT(*) FROM sessions s
          WHERE s.tutor_id = tp.user_id AND s.status = 'completed'
       ), 0);
