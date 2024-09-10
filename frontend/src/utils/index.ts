import dayjs from 'dayjs';

export function formatDate(date: string) {
  const _date = dayjs(date);
  const year = _date.get('year');
  const nowYear = new Date().getFullYear();
  if (year === nowYear) {
    return _date.format('MM-DD HH:mm:ss');
  }
  return _date.format('YYYY-MM-DD HH:mm:ss');
}
