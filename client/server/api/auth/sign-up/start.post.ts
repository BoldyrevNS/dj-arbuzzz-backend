import { z } from 'zod';

const bodySchema = z.object({
	email: z.email(),
});

export default defineEventHandler(async (event) => {
	console.log('sign-up-start called');
	const { email } = await readValidatedBody(event, bodySchema.parse);
	const config = useRuntimeConfig(event);

	const url = `${config.public.apiBase}/sign-up/start`;
	console.log('Calling backend URL:', url);

	try {
		const res = await fetch(url, {
			method: 'POST',
			headers: {
				'Content-Type': 'application/json',
			},
			body: JSON.stringify({ email }),
		});

		console.log('Backend response status:', res.status);

		if (!res.ok) {
			const errorData = await res.json().catch(() => ({}));
			console.log('Backend error:', errorData);
			throw createError({
				statusCode: res.status,
				message: errorData.message || 'Failed to start sign up',
			});
		}

		return await res.json();
	}
	catch (error: any) {
		console.log('Caught error:', error);

		if (error.statusCode) {
			throw error;
		}

		throw createError({
			statusCode: 500,
			message: error.message || 'Failed to start sign up',
		});
	}
});
