import './style.scss';

import dayjs from 'dayjs';
import { useMemo } from 'react';
import ReactMarkdown from 'react-markdown';
import rehypeSanitaze from 'rehype-sanitize';

import { useI18nContext } from '../../../../i18n/i18n-react';
import { Card } from '../../../../shared/components/layout/Card/Card';
import { EnrollmentStepIndicator } from '../../components/EnrollmentStepIndicator/EnrollmentStepIndicator';
import { useEnrollmentStore } from '../../hooks/store/useEnrollmentStore';

export const WelcomeStep = () => {
  const { LL, locale } = useI18nContext();
  const sessionEnd = useEnrollmentStore((state) => state.sessionEnd);

  const markdown = useMemo(() => {
    let timeEnd: string;
    if (sessionEnd) {
      const now = dayjs();
      const endDay = dayjs(sessionEnd);
      const diff = endDay.diff(now, 'minute');
      timeEnd = dayjs.duration(diff, 'minutes').locale(locale).humanize();
    }
    timeEnd = 'not set';

    return LL.pages.enrollment.steps.welcome.explanation({ time: timeEnd });
  }, [LL.pages.enrollment.steps.welcome, sessionEnd, locale]);

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
