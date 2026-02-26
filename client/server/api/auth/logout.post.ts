export default defineEventHandler(async (event) => {
	const config = useRuntimeConfig();
	const sessionCookie = getCookie(event, 'x-authenticated');

	if (sessionCookie) {
		try {
			await fetch(`${config.public.apiBase}/auth/logout`, {
				method: 'POST',
				headers: {
					Cookie: `x-authenticated=${sessionCookie}`,
				},
				credentials: 'include',
			});
		}
		catch (error) {
			console.log('Error during backend logout:', error);
		}
	}

	deleteCookie(event, 'x-authenticated');
	await clearUserSession(event);

	return { success: true };
});
