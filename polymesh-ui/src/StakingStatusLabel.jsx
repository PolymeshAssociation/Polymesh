const React = require('react');
const { Icon, Label } = require('semantic-ui-react');

import { ReactiveComponent } from 'oo7-react';
import { runtime } from 'oo7-substrate';
import { Pretty } from './Pretty';

export class StakingStatusLabel extends ReactiveComponent {
	constructor () {
		super (['id'], {
			info: ({id}) => runtime.staking.info(id).map(info => info && Object.assign({exposure: runtime.staking.exposureOf(info.ledger.stash)}, info))
		})
	}
	render () {
		if (!this.state.info) {
			return <Label><Icon name='times'/>Not staked</Label>
		}
		let info = this.state.info

		let intends = info.role.validator
			? 'validate'
			: info.role.nominator
			? 'nominate'
			: 'stop'
			
		let detail = (<Label.Detail>
			{<span><Pretty value={info.ledger.total}/> bonded</span>}
			{info.ledger.unlocking.length > 0
				? <span style={{marginLeft: '1em'}}> of which <Pretty value={info.ledger.unlocking.map(n => n.value).reduce((p, n) => p.add(n))}/> unbonding</span>
				: ''
			}
		</Label.Detail>)


		if (info.exposure.validating) {
			return <Label>
				<Icon name='certificate'/>
				Validating
				{intends != 'validate' ? <span>; intends to {intends}</span> : ''}
				{detail}
			</Label>
		}
		if (info.exposure.nominating) {
			return <Label>
				<Icon name='play'/>
				Nominating
				{intends != 'nominate' ? <span>; intends to {intends}</span> : ''}
				{detail}
			</Label>
		}
		return <Label>
			<Icon name='pause'/>
			Idle
			{intends != 'stop' ? <span>; intends to {intends}</span> : ''}
			{detail}
		</Label>
	}
}
