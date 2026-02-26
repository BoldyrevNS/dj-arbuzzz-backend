<script setup lang="ts">
import BackgroundLayout from './BackgroundLayout.vue';
</script>

<template>
	<BackgroundLayout>
		<div class="defaultLayout">
			<div class="headerContainer">
				<Header />
			</div>
			<div class="pageContainer">
				<slot />
			</div>
		</div>
	</BackgroundLayout>
</template>

<style lang="scss" scoped>
  .defaultLayout {
    height: 100%;
    width: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    align-items: center;
  }
  .headerContainer {
    height: 70px;
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .pageContainer {
    flex: 1;
    width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }
</style>
