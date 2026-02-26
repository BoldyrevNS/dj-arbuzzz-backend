<script lang="ts" setup>
async function handleClick() {
	await $fetch('/api/auth/logout', {
		method: 'POST',
	});
	await navigateTo('/auth/sign-in');
}
</script>

<template>
	<div class="index">
		<PlayerWidget />
	</div>
</template>

<style lang="scss" scoped>
.index {
  overflow: auto;
	display: flex;
	gap: 20px;
	flex-direction: column;
	align-items: center;
	justify-content: center;
	height: 100%;
}
</style>
