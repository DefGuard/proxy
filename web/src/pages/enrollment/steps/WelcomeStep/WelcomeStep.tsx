import './style.scss';

import dayjs from 'dayjs';
import { useCallback, useEffect, useMemo, useState } from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeSanitaze from 'rehype-sanitize';
import { timer } from 'rxjs';
import { shallow } from 'zustand/shallow';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';

export const WelcomeStep = () => {
  const { LL, locale } = useI18nContext();
  const [timeLeft, setTimeLeft] = useState<string | undefined>();
  const sessionEnd = useEnrollmentStore((state) => state.sessionEnd);

  const [nextSubject, next] = useEnrollmentStore(
    (state) => [state.nextSubject, state.nextStep],
    shallow,
  );
  const markdown = useMemo(() => {
    let timeEnd = 'not set';
    if (sessionEnd) {
      const now = dayjs();
      const endDay = dayjs(sessionEnd);
      const diff = endDay.diff(now, 'minute');
      timeEnd = dayjs.duration(diff, 'minutes').locale(locale).humanize();
    }

  const updateTimeLeft = useCallback(
    (endDate: string) => {
      const endDay = dayjs(endDate);
      const now = dayjs();
      const diff = endDay.diff(now, 'minute');
      setTimeLeft(dayjs.duration(diff, 'minutes').locale(locale).humanize());
    },
    [locale],
  );

  const markdown = useMemo(() => {
    return LL.pages.enrollment.steps.welcome.explanation({
      time: timeLeft ?? 'not set',
    });
  }, [LL.pages.enrollment.steps.welcome, timeLeft]);

  useEffect(() => {
    const sub = nextSubject.subscribe(() => {
      next();
    });
    return () => {
      sub.unsubscribe();
    };
  }, [next, nextSubject]);

  useEffect(() => {
    if (sessionEnd) {
      const sub = timer(0, 1000 * 60).subscribe(() => updateTimeLeft(sessionEnd));
      return () => {
        sub.unsubscribe();
      };
    }
  }, [sessionEnd, updateTimeLeft, locale]);

  return (
    <>
      <Card id="enrollment-welcome-card">
        <EnrollmentStepIndicator />
        <h3>{LL.pages.enrollment.steps.welcome.title({ name: 'placeholder' })}</h3>
        <div className="explenation">
          <ReactMarkdown rehypePlugins={[rehypeSanitaze]}>{markdown}</ReactMarkdown>
        </div>
      </Card>
    </>
  );
};
