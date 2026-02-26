<script setup lang="ts">
import Logo from '~/assets/icons/app-icon.svg';

const { loggedIn } = useUserSession();

async function handleLogout() {
	await $fetch('/api/auth/logout', {
		method: 'POST',
	});
	await navigateTo('/auth/sign-in');
}
</script>

<template>
	<div class="header">
		<RouterLink
			to="/"
			class="logoContainer"
		>
			<Logo />
		</RouterLink>
		<Button
			v-if="loggedIn"
			@click="handleLogout"
		>
			Выход
		</Button>
	</div>
</template>

<style lang="scss" scoped>
  .header {
    width: 1400px;
    height: 100%;
    background-color: $glass_item_background;
    backdrop-filter: $blur;
    -webkit-backdrop-filter: $blur;
    border-bottom: $glass_border;
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 30px;
    border-radius: 0 0 $border_radius $border_radius;
  }

  .appName {
    display: flex;
    align-items: center;
    justify-self: flex-start;
    gap: 10px;
  }

  .logoContainer {
    width: 50px;
    height: 50px;
    transition: $transition;
    &:hover {
      transform: rotate(10deg);
      cursor: pointer;
    }
  }
</style>
