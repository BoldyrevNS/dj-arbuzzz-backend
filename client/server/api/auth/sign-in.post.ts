import { z } from 'zod';

const bodySchema = z.object({
	email: z.string().min(3),
	password: z.string().min(8),
});

export default defineEventHandler(async (event) => {
	const { email, password } = await readValidatedBody(event, bodySchema.parse);
	const config = useRuntimeConfig();
	console.log(`${config.public.apiBase}/auth/sign-in`);

	const response = await fetch(`${config.public.apiBase}/auth/sign-in`, {
		body: JSON.stringify({ email, password }),
		method: 'POST',
		headers: {
			'Content-Type': 'application/json',
		},
	});

	if (!response.ok) {
		const errorText = await response.text();
		console.error('Sign in failed:', response.status, errorText);

		throw createError({
			statusCode: response.status,
			message: errorText || 'Failed to sign in',
		});
	}

	const cookies = response.headers.get('set-cookie');
	if (cookies) {
		const cookieArray = cookies.split(',').map(c => c.trim());
		for (const cookie of cookieArray) {
			if (cookie.startsWith('x-authenticated=')) {
				const [nameValue] = cookie.split(';');
				const [name, value] = nameValue.split('=');

				setCookie(event, name, value, {
					httpOnly: true,
					secure: process.env.NODE_ENV === 'production',
					sameSite: 'lax',
					path: '/',
				});
			}
		}
	}

	await setUserSession(event, {
		user: {
			authenticated: true,
		},
	});

	return { success: true };
});
