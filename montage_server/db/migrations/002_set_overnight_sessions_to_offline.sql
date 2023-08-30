UPDATE sessions
SET kind = 'offline'
WHERE strftime('%Y-%m-%d', start_time) != strftime('%Y-%m-%d', end_time)
AND kind = 'break';
