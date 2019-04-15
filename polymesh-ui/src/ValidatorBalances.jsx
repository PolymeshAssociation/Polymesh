import { ReactiveComponent } from 'oo7-react';
import { pretty } from 'oo7-substrate';
import Identicon from 'polkadot-identicon';

export class ValidatorBalances extends ReactiveComponent {
	constructor () {
		super(["value", "className"])
	}
	
	render() {
		if (!this.state.value) return (<div/>)
		return (<div className={this.state.className || 'validator-balances'} name={this.props.name}>
			{this.state.value.map((v, i) => (<div key={i} className="validator-balance">
				<div className="validator"><Identicon account={v.who} size={52}/></div>
				<div className="nominators">{v.nominators.map(a => <Identicon account={a} size={24}/>)}</div>
				<div className="AccountId">{pretty(v.who).substr(0, 8) + '…'}</div>
				<div className="Balance">{pretty(v.balance)}</div>
				{
					(v.otherBalance > 0
						? <div className="paren">{' (incl. ' + pretty(v.otherBalance) + ' nominated)'}</div>
						: null
					)
				}
			</div>))}
		</div>)
	}
}
